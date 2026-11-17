use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Output};

fn run(path: &Path) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_quality"))
        .arg(path)
        .output()
}

#[test]
fn short_functions_pass() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("a.rs"), "fn short() { a(); }").unwrap();

    let output = run(directory.path()).unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"");
}

#[test]
fn a_long_function_is_printed_and_fails() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("a.rs");
    fs::write(&file, "fn long() { a(); b(); c(); d(); e(); f(); }").unwrap();

    let output = run(directory.path()).unwrap();

    assert_eq!(output.status.code(), Some(1));
    let printed = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        printed,
        format!("{}:1: long has 6 statements\n", file.display())
    );
}

#[test]
fn a_broken_file_is_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("broken.rs");
    fs::write(&file, "fn (").unwrap();

    let output = run(&file).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with(&file.display().to_string())
    );
}
