use std::fs::File;
use std::io::Write;

struct CSource {
    input_path: String,
    output_path: String,
}

struct Sources {
    sources: Vec<CSource>,
}

impl Sources {
    pub fn new() -> Self {
        return Sources {
            sources: Vec::new(),
        }
    }

    pub fn push_source(&mut self, input_path: String) {
        let output_path = input_path.replace('/', "_") + ".o";
        self.sources.push(CSource { input_path, output_path });
    }
}

fn make_ninja_file(sources: Sources) -> Result<(), std::io::Error> {
    let mut f = File::create("build.ninja")?;

    writeln!(f, "builddir = build")?;
    
    writeln!(f, "cc = gcc")?;
    writeln!(f, "cflags = -g -Wall ")?;

    writeln!(f, "")?;

    writeln!(f, "rule cc")?;
    writeln!(f, "  command = $cc -MMD -MT $out -MF $out.d $cflags -c $in -o $out")?;
    writeln!(f, "  depfile = $out.d")?;
    writeln!(f, "  deps = gcc")?;

    writeln!(f, "")?;

    for source in &sources.sources {
        writeln!(f, "build $builddir/{}: cc {}", source.output_path, source.input_path)?;
    }

    Ok(())
}

fn main() {
    let mut sources = Sources::new();

    sources.push_source("src/pony_main.c".into());

    match make_ninja_file(sources) {
        Ok(()) => {},
        Err(error) => {
            eprintln!("Failed to write build.ninja: {}", error);
        }
    }
}
