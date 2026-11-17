//! Making commits and reading the last message.

use std::path::Path;

use super::invalid;
use crate::{Error, git};

pub fn commit(repository: &Path, message: &str) -> Result<(), Error> {
    if message.trim().is_empty() {
        return Err(invalid("the commit message is empty".to_owned()));
    }
    git::write_with_input(repository, &["commit", "-F", "-"], message).map(drop)
}

/// Replaces the last commit. An empty message keeps the old message.
pub fn amend(repository: &Path, message: &str) -> Result<(), Error> {
    if message.trim().is_empty() {
        return git::write(repository, &["commit", "--amend", "--no-edit"]).map(drop);
    }
    git::write_with_input(repository, &["commit", "-F", "-", "--amend"], message).map(drop)
}

pub fn last_message(repository: &Path) -> Result<String, Error> {
    let message = git::read(repository, &["log", "-1", "--format=%B"])?;
    Ok(message.trim_end().to_owned())
}
