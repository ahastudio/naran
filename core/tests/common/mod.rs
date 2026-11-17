//! Helpers shared by the test files. Each file uses only some of them.
#![allow(dead_code, reason = "each test file uses only some of the helpers")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn git(repository: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Naran")
        .env("GIT_AUTHOR_EMAIL", "naran@example.com")
        .env("GIT_COMMITTER_NAME", "Naran")
        .env("GIT_COMMITTER_EMAIL", "naran@example.com")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// A new repository in a temporary folder that is removed when it is dropped.
pub fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    init(directory.path());
    directory
}

pub fn init(directory: &Path) -> PathBuf {
    fs::create_dir_all(directory).unwrap();
    git(directory, &["init", "-q", "-b", "main"]);
    git(directory, &["config", "user.name", "Naran"]);
    git(directory, &["config", "user.email", "naran@example.com"]);
    directory.to_path_buf()
}

pub fn write(repository: &Path, file: &str, content: &str) {
    fs::write(repository.join(file), content).unwrap();
}

pub fn commit_file(repository: &Path, file: &str, content: &str, message: &str) {
    write(repository, file, content);
    git(repository, &["add", "--", file]);
    git(repository, &["commit", "-q", "-m", message]);
}

/// Creates a bare remote and two clones of it.
pub fn with_remote(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let remote = root.join("remote.git");
    fs::create_dir_all(&remote).unwrap();
    git(&remote, &["init", "-q", "--bare", "-b", "main"]);
    let (first, second) = (pushed_clone(root, &remote), clone(root, &remote));
    (remote, first, second)
}

/// A repository with one commit, pushed to `remote` as `main`.
fn pushed_clone(root: &Path, remote: &Path) -> PathBuf {
    let first = init(&root.join("first"));
    git(
        &first,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    commit_file(&first, "a.txt", "one\n", "첫째");
    git(&first, &["push", "-q", "-u", "origin", "main"]);
    first
}

fn clone(root: &Path, remote: &Path) -> PathBuf {
    let second = root.join("second");
    git(
        root,
        &[
            "clone",
            "-q",
            remote.to_str().unwrap(),
            second.to_str().unwrap(),
        ],
    );
    git(&second, &["config", "user.name", "Naran"]);
    git(&second, &["config", "user.email", "naran@example.com"]);
    second
}

/// Compares the tips of `from` and `to`.
pub fn compare<'a>(from: &'a str, to: &'a str) -> naran_core::Comparison<'a> {
    naran_core::Comparison { from, to }
}

pub fn tag<'a>(
    name: &'a str,
    target: Option<&'a str>,
    message: Option<&'a str>,
) -> naran_core::ops::NewTag<'a> {
    naran_core::ops::NewTag {
        name,
        target,
        message,
    }
}
