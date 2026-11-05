//! Writes the generated TypeScript types and JSON Schema into their tracked
//! files, or with `--check` fails if they are out of date.

use std::path::Path;
use std::process::ExitCode;
use std::{env, fs};

use buwiz_agent::generate::{SCHEMA_PATH, TYPESCRIPT_PATH, schema, typescript};

fn main() -> ExitCode {
    let check = env::args().nth(1).as_deref() == Some("--check");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut stale = false;
    for (path, text) in [(TYPESCRIPT_PATH, typescript()), (SCHEMA_PATH, schema())] {
        let file = root.join(path);
        if !check {
            if let Err(error) = fs::write(&file, text) {
                eprintln!("{path}: {error}");
                return ExitCode::FAILURE;
            }
            println!("wrote {path}");
        } else if fs::read_to_string(&file).ok().as_deref() != Some(text.as_str()) {
            eprintln!("{path} is out of date: run cargo run -p buwiz-agent --bin buwiz-agent-gen");
            stale = true;
        }
    }
    if stale {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
