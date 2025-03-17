use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::rc::Rc;

pub mod lexer;

struct CSource {
    input_path: String,
    output_path: String,
}

struct Sources {
    c_sources: Vec<CSource>,
}

enum ArtifactKind {
    Binary,
    StaticLib,
    DynLib,
}

/// Artifacts are the main thing that the pony build system is trying to build.
/// An artifact can be a binary application or a static or dynamic library. They
/// may also eventually be other things, i.e. just general targets for the build
/// system.
struct Artifact {
    name: Rc<String>,
    output: Rc<String>,
    sources: Vec<Rc<Sources>>,
    kind: ArtifactKind,
}

impl Artifact {
    pub fn new(name: String, kind: ArtifactKind) -> Artifact {
        let output = Rc::new(format!("build/{name}"));
        Artifact {
            name: Rc::new(name),
            output,
            sources: Vec::new(),
            kind
        }
    }
}

struct Scope {
    artifacts: Vec<Rc<Artifact>>,
    variables: HashMap<String, Object>,
    scopes: Vec<Rc<Scope>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            artifacts: Vec::new(),
            variables: HashMap::new(),
            scopes: Vec::new()
        }
    }
}

#[derive(Clone)]
enum Object {
    Artifact(Rc<Artifact>),
    Sources(Rc<Sources>),
    String(Rc<String>),
    Scope(Rc<Scope>),
}

trait Lookup {
    fn lookup(&self, name: &str) -> Option<Object>;
}

impl Lookup for Artifact {
    fn lookup(&self, name: &str) -> Option<Object> {
        Some(match name {
            //"sources" => Object::Sources(Rc::clone(&self.sources)),
            "output" => Object::String(Rc::clone(&self.output)),
            "name" => Object::String(Rc::clone(&self.name)),
            _ => return None
        })
    }
}

impl Lookup for Scope {
    fn lookup(&self, name: &str) -> Option<Object> {
        self.variables.get(name).cloned()
    }
}

impl Sources {
    pub fn new() -> Sources {
        return Sources {
            c_sources: Vec::new(),
        };
    }

    pub fn push_c_source(&mut self, input_path: String) {
        let output_path = input_path.replace('/', "_") + ".o";
        self.c_sources.push(CSource { input_path, output_path });
    }
}

fn genlist_object_files(f: &mut File, artifact: &Artifact) -> Result<(), std::io::Error> {
    for sources in &artifact.sources {
        for c_source in &sources.c_sources {
            write!(f, "$builddir/{} ", c_source.output_path)?;
        }
    }

    Ok(())
}

fn genrules_artifact(f: &mut File, artifact: &Artifact) -> Result<(), std::io::Error> {
    let binary_ext = ".exe";
    
    for sources in &artifact.sources {
        for c_source in &sources.c_sources {
            writeln!(f, "build $builddir/{}: cc {}", c_source.output_path, c_source.input_path)?;
        }
    }

    match artifact.kind {
        ArtifactKind::Binary => {
            write!(f, "build {}{}: link ", artifact.output, binary_ext)?;
            genlist_object_files(f, artifact)?;
            writeln!(f, "\n")?;
        },
        ArtifactKind::StaticLib => {
            write!(f, "build {}: ar ", artifact.output)?;
            genlist_object_files(f, artifact)?;
            writeln!(f, "\n")?;
        },
        ArtifactKind::DynLib => {
            write!(f, "build {}: linkso ", artifact.output)?;
            genlist_object_files(f, artifact)?;
            writeln!(f, "\n")?;
        },
    }

    Ok(())
}

fn genrules_scope(f: &mut File, scope: &Scope) -> Result<(), std::io::Error> {
    for artifact in &scope.artifacts {
        genrules_artifact(f, artifact)?;
    }

    Ok(())
}

fn make_ninja_file(top_scope: &Scope) -> Result<(), std::io::Error> {
    let mut f = File::create("build.ninja")?;

    writeln!(f, "builddir = build")?;
    
    writeln!(f, "cc = gcc")?;
    writeln!(f, "cflags = -g -Wall ")?;

    writeln!(f, "")?;

    writeln!(f, "rule cc")?;
    writeln!(f, "  command = $cc -MMD -MT $out -MF $out.d $cflags -c $in -o $out")?;
    writeln!(f, "  depfile = $out.d")?;
    writeln!(f, "  deps = gcc")?;
    writeln!(f, "  description = CC      $in")?;
    writeln!(f, "")?;

    writeln!(f, "rule ar")?;
    writeln!(f, "  command = $ar rcs $out $in")?;
    writeln!(f, "  description = AR      $out")?;
    writeln!(f, "")?;

    writeln!(f, "rule link")?;
    writeln!(f, "  command = $cc -o $out $in $ldflags")?;
    writeln!(f, "  description = LINK    $out")?;
    writeln!(f, "")?;

    writeln!(f, "rule linkso")?;
    writeln!(f, "  command = $cc -shared -o $out $in $ldflags")?;
    writeln!(f, "  description = LINKSO  $out")?;
    writeln!(f, "")?;

    genrules_scope(&mut f, top_scope);

    Ok(())
}

fn main() {
    let mut scope = Scope::new();
    let mut output = Artifact::new("ponygame-runner".into(), ArtifactKind::Binary);
    let mut sources = Sources::new();

    sources.push_c_source("src/pony_main.c".into());

    output.sources.push(Rc::new(sources));

    scope.artifacts.push(Rc::new(output));

    match make_ninja_file(&scope) {
        Ok(()) => {},
        Err(error) => {
            eprintln!("Failed to write build.ninja: {}", error);
        }
    }
}
