//! Listing, switching, creating, deleting and renaming branches.

use std::path::Path;

use serde::Serialize;

use super::check_name;
use crate::{Error, git};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    pub name: String,
    pub remote: bool,
    pub current: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

pub fn branches(repository: &Path) -> Result<Vec<Branch>, Error> {
    let output = git::read(
        repository,
        &[
            "for-each-ref",
            "--format=%(HEAD)%1f%(refname)%1f%(refname:short)%1f%(upstream:short)%1f%(upstream:track,nobracket)",
            "refs/heads",
            "refs/remotes",
        ],
    )?;
    Ok(output.lines().filter_map(parse_branch).collect())
}

fn parse_branch(line: &str) -> Option<Branch> {
    let (head, full, name, upstream, track) = branch_fields(line)?;
    if full.ends_with("/HEAD") {
        return None;
    }
    Some(Branch {
        name: name.to_owned(),
        remote: full.starts_with("refs/remotes/"),
        current: head == "*",
        upstream: (!upstream.is_empty()).then(|| upstream.to_owned()),
        ahead: Divergence::Ahead.count_in(track),
        behind: Divergence::Behind.count_in(track),
    })
}

type BranchFields<'a> = (&'a str, &'a str, &'a str, &'a str, &'a str);

/// HEAD mark, full ref name, short name, upstream, and track text.
fn branch_fields(line: &str) -> Option<BranchFields<'_>> {
    match line.split('\x1f').collect::<Vec<_>>()[..] {
        [head, full, name, upstream, track, ..] => Some((head, full, name, upstream, track)),
        _ => None,
    }
}

/// How a branch differs from its upstream.
#[derive(Debug, Clone, Copy)]
enum Divergence {
    Ahead,
    Behind,
}

impl Divergence {
    /// Reads the number of commits from track text like `ahead 2, behind 1`.
    fn count_in(self, track: &str) -> u32 {
        let word = match self {
            Self::Ahead => "ahead ",
            Self::Behind => "behind ",
        };
        track
            .split(", ")
            .find_map(|part| part.strip_prefix(word)?.trim().parse().ok())
            .unwrap_or(0)
    }
}

pub fn switch(repository: &Path, name: &str) -> Result<(), Error> {
    check_name(name)?;
    git::write(repository, &["switch", name]).map(drop)
}

/// Checks out a commit, leaving HEAD detached.
pub fn switch_detached(repository: &Path, hash: &str) -> Result<(), Error> {
    crate::check_hash(hash)?;
    git::write(repository, &["switch", "--detach", hash]).map(drop)
}

/// Creates a branch at `start`, or at HEAD without one, and stays on the
/// current branch.
pub fn create_branch(repository: &Path, name: &str, start: Option<&str>) -> Result<(), Error> {
    check_name(name)?;
    write_from(repository, &["branch", name], start)
}

/// Creates a branch like [`create_branch`] and switches to it.
pub fn create_and_switch(repository: &Path, name: &str, start: Option<&str>) -> Result<(), Error> {
    check_name(name)?;
    write_from(repository, &["switch", "-c", name], start)
}

/// Runs `command`, followed by the start point when there is one.
fn write_from(repository: &Path, command: &[&str], start: Option<&str>) -> Result<(), Error> {
    let start = start
        .map(|name| check_name(name).map(|()| name))
        .transpose()?;
    let arguments: Vec<&str> = command.iter().copied().chain(start).collect();
    git::write(repository, &arguments).map(drop)
}

/// Deletes a branch. Git refuses when its commits are not merged yet.
pub fn delete_branch(repository: &Path, name: &str) -> Result<(), Error> {
    check_name(name)?;
    git::write(repository, &["branch", "-d", name]).map(drop)
}

/// Deletes a branch even when its commits are not merged, losing them.
pub fn force_delete_branch(repository: &Path, name: &str) -> Result<(), Error> {
    check_name(name)?;
    git::write(repository, &["branch", "-D", name]).map(drop)
}

/// Number of commits missing from the current branch. Used to warn before deleting a branch.
pub fn unmerged_count(repository: &Path, name: &str) -> Result<u32, Error> {
    check_name(name)?;
    let range = format!("HEAD..{name}");
    let output = git::read(repository, &["rev-list", "--count", &range])?;
    Ok(output.trim().parse().unwrap_or(0))
}

pub fn rename_branch(repository: &Path, old: &str, new: &str) -> Result<(), Error> {
    check_name(old)?;
    check_name(new)?;
    git::write(repository, &["branch", "-m", old, new]).map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_that_are_cut_short_are_skipped() {
        assert_eq!(branch_fields("*\x1fa\x1fb\x1fc"), None);
        assert!(branch_fields("*\x1fa\x1fb\x1fc\x1fd").is_some());
        assert!(parse_branch("*\x1fa").is_none());
    }

    #[test]
    fn parses_branches() {
        let line = "*\x1frefs/heads/main\x1fmain\x1forigin/main\x1fahead 2, behind 1";
        let branch = parse_branch(line).unwrap();
        assert!(branch.current && !branch.remote);
        assert_eq!((branch.ahead, branch.behind), (2, 1));
        assert_eq!(branch.upstream.as_deref(), Some("origin/main"));

        let behind_only = parse_branch(" \x1frefs/heads/x\x1fx\x1fo/x\x1fbehind 3").unwrap();
        assert_eq!((behind_only.ahead, behind_only.behind), (0, 3));

        let remote = parse_branch(" \x1frefs/remotes/origin/x\x1forigin/x\x1f\x1f").unwrap();
        assert!(remote.remote && remote.upstream.is_none());

        assert_eq!(
            parse_branch(" \x1frefs/remotes/origin/HEAD\x1forigin\x1f\x1f"),
            None
        );
        assert_eq!(parse_branch("broken"), None);
    }
}
