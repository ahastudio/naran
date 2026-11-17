//! Merge, rebase, cherry-pick and revert, and finishing them after a conflict.

use std::path::Path;

use serde::Deserialize;

use super::{check_name, invalid};
use crate::{Error, Operation, git, summarize};

pub fn merge(repository: &Path, name: &str) -> Result<(), Error> {
    check_name(name)?;
    git::write(repository, &["merge", name]).map(drop)
}

pub fn rebase(repository: &Path, onto: &str) -> Result<(), Error> {
    check_name(onto)?;
    git::write(repository, &["rebase", onto]).map(drop)
}

pub fn cherry_pick(repository: &Path, hash: &str) -> Result<(), Error> {
    crate::check_hash(hash)?;
    git::write(repository, &["cherry-pick", hash]).map(drop)
}

pub fn revert(repository: &Path, hash: &str) -> Result<(), Error> {
    crate::check_hash(hash)?;
    git::write(repository, &["revert", "--no-edit", hash]).map(drop)
}

fn operation_command(operation: Operation) -> &'static str {
    match operation {
        Operation::Merge => "merge",
        Operation::Rebase => "rebase",
        Operation::CherryPick => "cherry-pick",
        Operation::Revert => "revert",
    }
}

pub fn abort_operation(repository: &Path) -> Result<(), Error> {
    let operation = in_progress(repository)?;
    git::write(repository, &[operation_command(operation), "--abort"]).map(drop)
}

/// Continues the operation in progress after conflicts are resolved and staged.
pub fn continue_operation(repository: &Path) -> Result<(), Error> {
    let arguments: Vec<&str> = match in_progress(repository)? {
        Operation::Merge => vec!["commit", "--no-edit"],
        other => vec![operation_command(other), "--continue"],
    };
    git::write(repository, &arguments).map(drop)
}

fn in_progress(repository: &Path) -> Result<Operation, Error> {
    summarize(repository)?
        .operation
        .ok_or_else(|| invalid("no operation in progress".to_owned()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Ours,
    Theirs,
}

/// Marks a conflicted file as resolved. With `side`, takes that side's content.
/// During a rebase, ours and theirs are swapped compared to intuition (a Git rule).
pub fn resolve(repository: &Path, file: &str, side: Option<Side>) -> Result<(), Error> {
    if let Some(side) = side {
        git::write(repository, &["checkout", side.flag(), "--", file])?;
    }
    git::write(repository, &["add", "--", file]).map(drop)
}

impl Side {
    fn flag(self) -> &'static str {
        match self {
            Self::Ours => "--ours",
            Self::Theirs => "--theirs",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operation_has_its_git_command() {
        let commands = [
            (Operation::Merge, "merge"),
            (Operation::Rebase, "rebase"),
            (Operation::CherryPick, "cherry-pick"),
            (Operation::Revert, "revert"),
        ];
        for (operation, command) in commands {
            assert_eq!(operation_command(operation), command);
        }
    }

    #[test]
    fn each_side_has_its_checkout_flag() {
        assert_eq!(Side::Ours.flag(), "--ours");
        assert_eq!(Side::Theirs.flag(), "--theirs");
    }
}
