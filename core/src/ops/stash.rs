//! Putting changes aside in stashes and bringing them back.

use std::path::Path;

use serde::Serialize;

use super::invalid;
use crate::{Error, git};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stash {
    pub name: String,
    pub message: String,
}

pub fn stash_list(repository: &Path) -> Result<Vec<Stash>, Error> {
    let output = git::read(repository, &["stash", "list", "--format=%gd%x1f%s"])?;
    Ok(output.lines().filter_map(parse_stash).collect())
}

fn parse_stash(line: &str) -> Option<Stash> {
    let (name, message) = line.split_once('\x1f')?;
    Some(Stash {
        name: name.to_owned(),
        message: message.to_owned(),
    })
}

/// What a stash does with files Git does not track yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Untracked {
    /// They stay in the working tree.
    Leave,
    /// They go into the stash too.
    Include,
}

pub fn stash_push(repository: &Path, message: &str, untracked: Untracked) -> Result<(), Error> {
    let untracked_flag = (untracked == Untracked::Include).then_some("--include-untracked");
    let message_flags = (!message.trim().is_empty()).then_some(["-m", message]);
    let flags = untracked_flag
        .into_iter()
        .chain(message_flags.into_iter().flatten());
    let arguments: Vec<&str> = ["stash", "push"].into_iter().chain(flags).collect();
    git::write(repository, &arguments).map(drop)
}

pub fn stash_apply(repository: &Path, name: &str) -> Result<(), Error> {
    check_stash(name)?;
    git::write(repository, &["stash", "apply", name]).map(drop)
}

pub fn stash_drop(repository: &Path, name: &str) -> Result<(), Error> {
    check_stash(name)?;
    git::write(repository, &["stash", "drop", name]).map(drop)
}

/// Accepts only names like `stash@{2}`, so a name cannot read as an option.
fn check_stash(name: &str) -> Result<(), Error> {
    let number = name
        .strip_prefix("stash@{")
        .and_then(|rest| rest.strip_suffix('}'));
    if number.is_some_and(is_number) {
        return Ok(());
    }
    Err(invalid(format!("not a stash name: {name}")))
}

fn is_number(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_that_are_cut_short_are_skipped() {
        assert_eq!(parse_stash("no separator"), None);
        assert_eq!(
            parse_stash("stash@{0}\x1fmessage").unwrap().message,
            "message"
        );
    }

    #[test]
    fn validates_stash_names() {
        assert!(check_stash("stash@{0}").is_ok());
        for name in ["stash@{}", "stash@{x}", "--all", "stash@{1}x"] {
            assert!(check_stash(name).is_err(), "{name}");
        }
    }
}
