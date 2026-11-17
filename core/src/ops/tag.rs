//! Listing, creating and deleting tags.

use std::path::Path;

use super::check_name;
use crate::{Error, git};

pub fn tags(repository: &Path) -> Result<Vec<String>, Error> {
    let output = git::read(repository, &["tag", "--list", "--sort=-creatordate"])?;
    Ok(output.lines().map(str::to_owned).collect())
}

/// A tag to create.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewTag<'a> {
    pub name: &'a str,
    /// The commit to tag. HEAD when there is none.
    pub target: Option<&'a str>,
    /// Makes the tag an annotated one.
    pub message: Option<&'a str>,
}

pub fn tag_create(repository: &Path, tag: &NewTag) -> Result<(), Error> {
    check_name(tag.name)?;
    let annotation = tag
        .message
        .filter(|message| !message.trim().is_empty())
        .map(|message| ["-a", "-m", message]);
    let target = crate::checked_hash(tag.target)?;
    let arguments: Vec<&str> = ["tag"]
        .into_iter()
        .chain(annotation.into_iter().flatten())
        .chain([tag.name])
        .chain(target)
        .collect();
    git::write(repository, &arguments).map(drop)
}

pub fn tag_delete(repository: &Path, name: &str) -> Result<(), Error> {
    check_name(name)?;
    git::write(repository, &["tag", "-d", name]).map(drop)
}
