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

pub fn file_diff(repository: &Path, file: &str, staging: Staging) -> Result<String, Error> {
    let cached = (staging == Staging::Staged).then_some("--cached");
    let arguments: Vec<&str> = ["diff", "--no-color", "--no-ext-diff"]
        .into_iter()
        .chain(cached)
        .chain(["--", file])
        .collect();
    git::read(repository, &arguments)
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
