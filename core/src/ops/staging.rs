//! Moving changes between the working tree and the index, and throwing them away.

use std::path::Path;

use super::{invalid, with_paths, write_with_paths};
use crate::patch::{self, Direction};
use crate::{Error, Staging, git};

pub fn stage(repository: &Path, paths: &[String]) -> Result<(), Error> {
    write_with_paths(repository, &["add"], paths)
}

pub fn unstage(repository: &Path, paths: &[String]) -> Result<(), Error> {
    let all = with_origins(repository, paths)?;
    let arguments = if has_commits(repository) {
        with_paths(&["restore", "--staged"], &all)?
    } else {
        with_paths(&["rm", "--cached", "-r", "-q"], paths)?
    };
    git::write(repository, &arguments).map(drop)
}

/// The paths and the old paths of the files renamed in the index among them, because
/// unstaging only the new path would leave the old one staged as deleted.
fn with_origins(repository: &Path, paths: &[String]) -> Result<Vec<String>, Error> {
    let origins = crate::changes(repository)?
        .into_iter()
        .filter(|change| paths.contains(&change.path))
        .filter(|change| change.index == 'R')
        .filter_map(|change| change.original);
    Ok(paths.iter().cloned().chain(origins).collect())
}

fn has_commits(repository: &Path) -> bool {
    git::read(repository, &["rev-parse", "--verify", "-q", "HEAD"]).is_ok()
}

/// Discards changes to tracked files. This cannot be undone, so callers must ask first.
pub fn discard(repository: &Path, paths: &[String]) -> Result<(), Error> {
    let restore = ["restore", "--source=HEAD", "--staged", "--worktree"];
    write_with_paths(repository, &restore, paths)
}

/// Deletes untracked files. This cannot be undone, so callers must ask first.
pub fn discard_untracked(repository: &Path, paths: &[String]) -> Result<(), Error> {
    write_with_paths(repository, &["clean", "-f", "-d"], paths)
}

/// Lines picked in the staged or unstaged diff of one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSelection<'a> {
    pub file: &'a str,
    /// The diff the lines were picked in.
    pub staging: Staging,
    /// Line numbers in the `file_diff` text split by `\n`.
    pub lines: &'a [usize],
}

/// Stages lines picked in the unstaged diff, or unstages lines picked in the
/// staged one.
pub fn stage_lines(repository: &Path, selection: &LineSelection) -> Result<(), Error> {
    let diff = crate::file_diff(repository, selection.file, selection.staging)?;
    let direction = selection.staging.direction();
    let patch = patch::build(&diff, selection.lines, direction)
        .ok_or_else(|| invalid("the selected lines contain no changes".to_owned()))?;
    git::write_with_input(repository, &apply_arguments(direction), &patch).map(drop)
}

fn apply_arguments(direction: Direction) -> Vec<&'static str> {
    let reverse_flag = (direction == Direction::Reverse).then_some("--reverse");
    ["apply", "--cached", "--recount", "--whitespace=nowarn"]
        .into_iter()
        .chain(reverse_flag)
        .chain(["-"])
        .collect()
}
