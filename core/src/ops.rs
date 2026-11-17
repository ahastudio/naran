//! Operations that change a repository, and listings such as branches, stashes and tags.

mod branch;
mod commit;
mod merge;
mod remote;
mod staging;
mod stash;
mod submodule;
mod tag;

use std::path::Path;

pub use branch::{
    Branch, branches, create_and_switch, create_branch, delete_branch, force_delete_branch,
    rename_branch, switch, switch_detached, unmerged_count,
};
pub use commit::{amend, commit, last_message};
pub use merge::{
    Side, abort_operation, cherry_pick, continue_operation, merge, rebase, resolve, revert,
};
pub use remote::{Outcome, fetch, force_push, pull, pull_safe, push};
pub use staging::{LineSelection, discard, discard_untracked, stage, stage_lines, unstage};
pub use stash::{Stash, Untracked, stash_apply, stash_drop, stash_list, stash_push};
pub use submodule::{Submodule, submodules};
pub use tag::{NewTag, tag_create, tag_delete, tags};

use crate::{Error, git, status};

fn invalid(message: String) -> Error {
    Error::InvalidArgument { message }
}

/// Branch, tag and remote names. Rejects names that would read as options or break arguments.
pub fn check_name(name: &str) -> Result<(), Error> {
    let bad = name.is_empty()
        || name.starts_with('-')
        || name
            .chars()
            .any(|letter| letter.is_whitespace() || letter.is_control());
    if bad {
        return Err(invalid(format!("name not allowed: {name:?}")));
    }
    Ok(())
}

/// Runs `git` with `prefix` and then the paths. Refuses an empty list of paths.
fn write_with_paths(repository: &Path, prefix: &[&str], paths: &[String]) -> Result<(), Error> {
    git::write(repository, &with_paths(prefix, paths)?).map(drop)
}

fn with_paths<'a>(prefix: &[&'a str], paths: &'a [String]) -> Result<Vec<&'a str>, Error> {
    if paths.is_empty() {
        return Err(invalid("no file selected".to_owned()));
    }
    let files = paths.iter().map(String::as_str);
    Ok(prefix.iter().copied().chain(["--"]).chain(files).collect())
}

/// Reads only the list of changed files from `git status`.
pub fn changed_paths(repository: &Path) -> Result<Vec<String>, Error> {
    let output = git::read(repository, &status::ARGUMENTS)?;
    Ok(status::parse_changes(&output)
        .into_iter()
        .map(|change| change.path)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_names() {
        for name in ["", "-x", "a b", "a\nb", "--force"] {
            assert!(check_name(name).is_err(), "{name:?}");
        }
        assert!(check_name("feature/한글").is_ok());
    }

    #[test]
    fn requires_paths() {
        assert!(with_paths(&["add"], &[]).is_err());
    }
}
