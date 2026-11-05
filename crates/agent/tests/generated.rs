//! The generated files in the repo match what the types generate now.

use std::fs;
use std::path::Path;

use buwiz_agent::generate::{SCHEMA_PATH, TYPESCRIPT_PATH, schema, typescript};

fn tracked(path: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path)).unwrap()
}

#[test]
fn the_typescript_is_up_to_date() {
    assert!(
        tracked(TYPESCRIPT_PATH) == typescript(),
        "{TYPESCRIPT_PATH} is out of date: run cargo run -p buwiz-agent --bin buwiz-agent-gen"
    );
}

#[test]
fn the_schema_is_up_to_date() {
    assert!(
        tracked(SCHEMA_PATH) == schema(),
        "{SCHEMA_PATH} is out of date: run cargo run -p buwiz-agent --bin buwiz-agent-gen"
    );
}

#[test]
fn no_generated_type_is_a_float() {
    assert!(!schema().contains("\"number\""));
    assert!(!typescript().contains("bigint"));
}
