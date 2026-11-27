//! Diffs of the working tree, of commits, and between two branches or commits.

use std::path::Path;

use serde::Serialize;

use crate::patch::Direction;
use crate::{Error, check_hash, git, ops};

/// Which changes of the working tree a diff shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staging {
    /// Changes in the working tree that are not in the index yet.
    Unstaged,
    /// Changes in the index, compared with the last commit.
    Staged,
}

impl Staging {
    /// Staging moves unstaged changes into the index; unstaging takes staged
    /// changes back out, which is the same patch applied in reverse.
    pub(crate) fn direction(self) -> Direction {
        match self {
            Self::Unstaged => Direction::Forward,
            Self::Staged => Direction::Reverse,
        }
    }
}

/// The diff of one file. A staged rename is paired with its old path, so
/// only the lines that changed show, not the whole file as added.
pub fn file_diff(repository: &Path, file: &str, staging: Staging) -> Result<String, Error> {
    let paths = diff_paths(repository, file, staging)?;
    let cached = (staging == Staging::Staged).then_some("--cached");
    let options = [
        "-c",
        "core.quotePath=false",
        "diff",
        "--no-color",
        "--no-ext-diff",
        "-M",
    ];
    let arguments: Vec<&str> = options
        .into_iter()
        .chain(cached)
        .chain(["--"])
        .chain(paths.iter().map(String::as_str))
        .collect();
    git::read(repository, &arguments)
}

/// The paths a diff of `file` names: for a staged rename also the old one,
/// because git can only pair the two when it sees both.
fn diff_paths(repository: &Path, file: &str, staging: Staging) -> Result<Vec<String>, Error> {
    if staging == Staging::Unstaged {
        return Ok(vec![file.to_owned()]);
    }
    let listing = ["diff", "--cached", "--name-status", "-M", "-z"];
    let output = git::read(repository, &listing)?;
    Ok(renamed_from(name_status_entries(&output), file)
        .into_iter()
        .chain([file.to_owned()])
        .collect())
}

fn renamed_from(entries: Vec<Vec<&str>>, file: &str) -> Option<String> {
    entries
        .into_iter()
        .find_map(|paths| match paths.as_slice() {
            [old, new] if *new == file => Some((*old).to_owned()),
            _ => None,
        })
}

/// The paths of each entry of `--name-status -z`: one, or two for a rename.
fn name_status_entries(output: &str) -> Vec<Vec<&str>> {
    let mut fields = output.split('\0').filter(|field| !field.is_empty());
    std::iter::from_fn(|| {
        let count = if fields.next()?.starts_with('R') {
            2
        } else {
            1
        };
        Some(fields.by_ref().take(count).collect())
    })
    .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitFile {
    pub path: String,
    pub status: char,
}

pub fn commit_files(repository: &Path, hash: &str) -> Result<Vec<CommitFile>, Error> {
    check_hash(hash)?;
    let output = git::read(
        repository,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-status",
            "--no-renames",
            "-r",
            "--root",
            "-z",
            hash,
        ],
    )?;
    Ok(parse_name_status(&output))
}

fn parse_name_status(output: &str) -> Vec<CommitFile> {
    let mut fields = output.split('\0').filter(|field| !field.is_empty());
    let mut files = Vec::new();
    while let (Some(status), Some(path)) = (fields.next(), fields.next()) {
        files.push(CommitFile {
            path: path.to_owned(),
            status: status.chars().next().unwrap_or('?'),
        });
    }
    files
}

/// The tips of two branches or commits, compared from one to the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comparison<'a> {
    pub from: &'a str,
    pub to: &'a str,
}

impl Comparison<'_> {
    fn check(&self) -> Result<(), Error> {
        ops::check_name(self.from)?;
        ops::check_name(self.to)
    }
}

/// The files that differ between the two tips.
pub fn compare_files(repository: &Path, comparison: &Comparison) -> Result<Vec<CommitFile>, Error> {
    comparison.check()?;
    let (from, to) = (comparison.from, comparison.to);
    let output = git::read(
        repository,
        &[
            "diff",
            "--name-status",
            "--no-renames",
            "-z",
            from,
            to,
            "--",
        ],
    )?;
    Ok(parse_name_status(&output))
}

pub fn compare_diff(
    repository: &Path,
    comparison: &Comparison,
    file: &str,
) -> Result<String, Error> {
    comparison.check()?;
    let (from, to) = (comparison.from, comparison.to);
    git::read(
        repository,
        &["diff", "--no-color", "--no-ext-diff", from, to, "--", file],
    )
}

/// One file as one commit changed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileInCommit<'a> {
    pub hash: &'a str,
    pub file: &'a str,
}

pub fn commit_diff(repository: &Path, target: &FileInCommit) -> Result<String, Error> {
    check_hash(target.hash)?;
    git::read(
        repository,
        &[
            "show",
            "--no-color",
            "--no-ext-diff",
            "--format=",
            target.hash,
            "--",
            target.file,
        ],
    )
}
