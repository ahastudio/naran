//! Naran core library. Knows nothing about the user interface.
#![cfg_attr(
    test,
    allow(clippy::too_many_lines, reason = "a test reads from top to bottom")
)]

pub mod batch;
pub mod blame;
mod blob;
mod diff;
pub mod git;
pub mod graph;
mod history;
pub mod log;
pub mod ops;
pub mod patch;
pub mod status;

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use blob::{BLOB_LIMIT, Side, blob, worktree_file};
pub use diff::{
    CommitFile, Comparison, FileInCommit, Staging, commit_diff, commit_files, compare_diff,
    compare_files, file_diff,
};
pub use history::{
    History, Page, Refs, Search, SearchKind, file_history, history, history_page, search,
};
pub use log::CommitDetail;
pub use status::{Change, Status};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Error {
    #[error("could not run git {command}: {message}")]
    Spawn { command: String, message: String },
    #[error("git {command} failed ({code:?}): {stderr}")]
    Command {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("git {command} timed out")]
    Timeout { command: String },
    #[error("invalid argument: {message}")]
    InvalidArgument { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Merge,
    Rebase,
    CherryPick,
    Revert,
}

/// Summary of one repository as shown on screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub path: PathBuf,
    pub name: String,
    pub status: Status,
    pub operation: Option<Operation>,
}

pub fn summarize(repository: &Path) -> Result<Summary, Error> {
    let arguments = [status::ARGUMENTS.as_slice(), &["--branch"]].concat();
    let (output, git_directory) = git::read(repository, &arguments).and_then(|output| {
        git::read(repository, &["rev-parse", "--absolute-git-dir"])
            .map(|directory| (output, directory))
    })?;

    Ok(Summary {
        path: repository.to_path_buf(),
        name: repository_name(repository),
        status: status::parse(&output),
        operation: operation_in_progress(Path::new(git_directory.trim())),
    })
}

/// The folder name, or the whole path for one that has no name (the root).
fn repository_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn operation_in_progress(git_directory: &Path) -> Option<Operation> {
    let markers = [
        ("rebase-merge", Operation::Rebase),
        ("rebase-apply", Operation::Rebase),
        ("MERGE_HEAD", Operation::Merge),
        ("CHERRY_PICK_HEAD", Operation::CherryPick),
        ("REVERT_HEAD", Operation::Revert),
    ];
    markers
        .into_iter()
        .find(|(marker, _)| git_directory.join(marker).exists())
        .map(|(_, operation)| operation)
}

pub fn changes(repository: &Path) -> Result<Vec<Change>, Error> {
    let output = git::read(repository, &status::ARGUMENTS)?;
    Ok(status::parse_changes(&output))
}

/// Message and meta information of one commit.
pub fn commit_detail(repository: &Path, hash: &str) -> Result<CommitDetail, Error> {
    check_hash(hash)?;
    let output = git::read(
        repository,
        &[
            "show",
            "-s",
            "--no-color",
            log::DETAIL_DATE,
            log::DETAIL_FORMAT,
            hash,
        ],
    )?;
    found_detail(log::parse_detail(&output), hash)
}

fn found_detail(detail: Option<CommitDetail>, hash: &str) -> Result<CommitDetail, Error> {
    detail.ok_or_else(|| Error::InvalidArgument {
        message: format!("unreadable commit: {hash}"),
    })
}

/// Passes an optional hash through after checking it.
pub(crate) fn checked_hash(hash: Option<&str>) -> Result<Option<&str>, Error> {
    hash.map(|hash| check_hash(hash).map(|()| hash)).transpose()
}

pub(crate) fn check_hash(hash: &str) -> Result<(), Error> {
    if !hash.is_empty() && hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(());
    }
    Err(Error::InvalidArgument {
        message: format!("not a commit hash: {hash}"),
    })
}

/// Author of each line. Without `revision` the working tree file is used.
pub fn blame(
    repository: &Path,
    file: &str,
    revision: Option<&str>,
) -> Result<Vec<blame::BlameLine>, Error> {
    let revision = checked_hash(revision)?;
    let arguments: Vec<&str> = ["blame", "--porcelain"]
        .into_iter()
        .chain(revision)
        .chain(["--", file])
        .collect();
    Ok(blame::parse(&git::read(repository, &arguments)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repository_is_named_by_its_folder_or_by_its_whole_path() {
        assert_eq!(repository_name(Path::new("/work/naran")), "naran");
        assert_eq!(repository_name(Path::new("/")), "/");
    }

    #[test]
    fn commit_output_that_cannot_be_read_is_an_error() {
        let error = found_detail(log::parse_detail("not a commit"), "abc123").unwrap_err();
        assert!(matches!(error, Error::InvalidArgument { message } if message.contains("abc123")));
    }
}
