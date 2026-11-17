//! Checks the rules of the coding guide that clippy does not check:
//!
//! - A function body has at most five statements. A statement inside an `if`,
//!   a `match` arm block, a loop or a plain block counts too; a closure is part
//!   of the statement it is written in.
//! - A function takes at most three parameters, not counting `self` and the
//!   window and context GPUI passes. Methods of a trait implementation are left
//!   out, because the trait decides their parameters.
//! - No parameter is a `bool` flag, and no two neighbouring parameters share a
//!   number or text type unless their order is their meaning, like `x, y`.
//! - A condition of an `if` or `while` fits on one line, does not mix `&&` with
//!   `||` and has at most two of them.
//! - A file has at most 300 lines, tests included.
//! - No two function bodies with three or more statements are the same.
//!
//! Tests are held only to the size of a file, as the guide leaves them out of
//! the rest: `#[test]` functions, `#[cfg(test)]` items, files named `tests.rs`
//! and files in `tests` folders.
#![cfg_attr(
    test,
    allow(clippy::too_many_lines, reason = "a test reads from top to bottom")
)]

mod conditions;
mod duplicates;
mod files;
mod parameters;
mod rules;
mod size;
mod statements;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub use statements::statements;

/// One place that breaks a rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub path: PathBuf,
    pub line: usize,
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let place = format!("{}:{}", self.path.display(), self.line);
        write!(formatter, "{place}: {}", self.message)
    }
}

/// A function, named where it starts.
#[derive(Debug, Clone)]
pub(crate) struct Place {
    path: PathBuf,
    line: usize,
    name: String,
}

impl Place {
    fn of(path: &Path, name: &syn::Ident) -> Self {
        let line = name.span().start().line;
        let name = name.to_string();
        Self {
            path: path.to_path_buf(),
            line,
            name,
        }
    }

    /// A finding about this function: `text` follows its name.
    fn message(&self, text: &str) -> Finding {
        Finding {
            path: self.path.clone(),
            line: self.line,
            message: format!("{} {text}", self.name),
        }
    }
}

/// What one file breaks, and its bodies to compare with the other files.
#[derive(Debug, Default)]
pub struct Checked {
    findings: Vec<Finding>,
    bodies: Vec<duplicates::Body>,
}

impl Checked {
    fn merge(checked: Vec<Self>) -> Vec<Finding> {
        let bodies: Vec<_> = checked.iter().flat_map(|one| one.bodies.clone()).collect();
        let mut findings: Vec<_> = checked.into_iter().flat_map(|one| one.findings).collect();
        findings.extend(duplicates::check(&bodies));
        findings.sort();
        findings
    }

    /// The findings of this file alone.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
}

/// Every finding in the files, and in every Rust file under the folders.
pub fn check(paths: &[PathBuf]) -> Result<Vec<Finding>, String> {
    let sources = files::sources(paths);
    let checked = sources.iter().map(|file| check_file(file));
    Ok(Checked::merge(checked.collect::<Result<_, _>>()?))
}

/// The findings in one file.
pub fn check_file(path: &Path) -> Result<Checked, String> {
    let failed = |error: &dyn fmt::Display| format!("{}: {error}", path.display());
    let source = fs::read_to_string(path).map_err(|error| failed(&error))?;
    let file = syn::parse_file(&source).map_err(|error| failed(&error))?;
    Ok(checked(path, &source, &file))
}

fn checked(path: &Path, source: &str, file: &syn::File) -> Checked {
    let mut checked = rules_for(path, file);
    checked.findings.extend(size::check(path, source));
    checked
}

/// A test file is held only to the size of a file.
fn rules_for(path: &Path, file: &syn::File) -> Checked {
    if files::is_test(path) {
        return Checked::default();
    }
    rules::check(path, file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(directory: &Path, name: &str, source: &str) -> PathBuf {
        let path = directory.join(name);
        fs::write(&path, source).unwrap();
        path
    }

    #[test]
    fn folders_are_walked_in_order_and_other_files_and_tests_are_skipped() {
        // A test file is still held to the size of a file.
        let directory = tempfile::tempdir().unwrap();
        let long = "fn long() { a(); b(); c(); d(); e(); f(); }";
        fs::create_dir_all(directory.path().join("nested/tests")).unwrap();
        write(directory.path(), "b.rs", long);
        write(directory.path(), "tests.rs", long);
        write(directory.path(), "nested/a.rs", &long.replace("f()", "g()"));
        write(directory.path(), "nested/tests/c.rs", long);
        write(directory.path(), "notes.txt", long);

        let found = check(&[directory.path().to_path_buf()]).unwrap();

        let files: Vec<_> = found
            .iter()
            .map(|finding| finding.path.file_name().unwrap())
            .collect();
        assert_eq!(files, ["b.rs", "a.rs"]);
    }

    #[test]
    fn the_same_body_in_two_files_is_found_once() {
        let directory = tempfile::tempdir().unwrap();
        let body = "{ a(); b(); c(); }";
        let first = write(directory.path(), "a.rs", &format!("fn one() {body}"));
        write(directory.path(), "b.rs", &format!("\nfn two() {body}"));

        let found = check(&[directory.path().to_path_buf()]).unwrap();

        let messages: Vec<_> = found.iter().map(ToString::to_string).collect();
        let expected = format!(
            "{}: two has the same body as one at {}:1",
            directory.path().join("b.rs:2").display(),
            first.display()
        );
        assert_eq!(messages, [expected]);
    }

    #[test]
    fn a_file_that_cannot_be_read_or_parsed_is_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let broken = write(directory.path(), "broken.rs", "fn (");

        assert!(
            check(std::slice::from_ref(&broken))
                .unwrap_err()
                .starts_with(&broken.display().to_string())
        );
        assert!(check_file(&directory.path().join("missing.rs")).is_err());
    }

    #[test]
    fn a_finding_is_shown_with_its_place() {
        let finding = Finding {
            path: "src/a.rs".into(),
            line: 3,
            message: "long has 6 statements".into(),
        };
        assert_eq!(finding.to_string(), "src/a.rs:3: long has 6 statements");
    }
}
