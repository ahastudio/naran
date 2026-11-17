//! Prints every place in the given files and folders that breaks a rule of
//! the coding guide, and fails when there is one.
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the report is the output"
)]

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match quality::check(&paths) {
        Ok(found) => report(&found),
        Err(error) => fail(&error),
    }
}

fn report(found: &[quality::Finding]) -> ExitCode {
    for finding in found {
        println!("{finding}");
    }
    ExitCode::from(u8::from(!found.is_empty()))
}

fn fail(error: &str) -> ExitCode {
    eprintln!("{error}");
    ExitCode::from(2)
}
