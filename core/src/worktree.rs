//! The other worktrees of a repository, read from `git worktree list`.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{Error, Summary, git, summarize_folder};

/// A worktree of the same repository as the folder being summarized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Worktree {
    pub path: PathBuf,
    pub locked: bool,
    pub prunable: bool,
    /// Read like a repository; an error when the folder is gone or unreadable.
    pub summary: Result<Summary, Error>,
}

/// One block of the porcelain list.
#[derive(Debug, PartialEq, Eq)]
struct Listed {
    path: PathBuf,
    bare: bool,
    locked: bool,
    prunable: bool,
}

/// The worktrees of `repository` except the folder itself. A listing that
/// failed leaves the list empty: worktrees are extra, so they never fail the
/// summary.
pub(crate) fn others(repository: &Path) -> Vec<Worktree> {
    let listing = git::read(repository, &["worktree", "list", "--porcelain"]);
    listing.map_or_else(|_| Vec::new(), |output| others_in(&output, repository))
}

fn others_in(output: &str, repository: &Path) -> Vec<Worktree> {
    let own = canonical(repository);
    let listed = parse(output).into_iter();
    let other = listed.filter(|listed| !listed.bare && canonical(&listed.path) != own);
    other.map(read_worktree).collect()
}

/// The path with links resolved, or as it is when the folder is gone.
fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// The message of the error for a worktree whose folder is gone, so a screen
/// can tell it from other failures.
pub const MISSING_FOLDER: &str = "worktree folder is missing";

fn read_worktree(listed: Listed) -> Worktree {
    let summary = if listed.path.exists() {
        summarize_folder(&listed.path)
    } else {
        Err(Error::InvalidArgument {
            message: MISSING_FOLDER.to_string(),
        })
    };
    Worktree {
        path: listed.path,
        locked: listed.locked,
        prunable: listed.prunable,
        summary,
    }
}

fn parse(output: &str) -> Vec<Listed> {
    output.split("\n\n").filter_map(parse_block).collect()
}

fn parse_block(block: &str) -> Option<Listed> {
    let mut lines = block.lines();
    let path = lines.next()?.strip_prefix("worktree ")?;
    let words: Vec<&str> = lines.collect();
    Some(Listed {
        path: PathBuf::from(path),
        bare: has(&words, "bare"),
        locked: has(&words, "locked"),
        prunable: has(&words, "prunable"),
    })
}

/// Whether a line starts with `word`, with or without a reason after it.
fn has(lines: &[&str], word: &str) -> bool {
    lines
        .iter()
        .any(|line| line.split(' ').next() == Some(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_porcelain_list() {
        let output = "worktree /repo\nHEAD abc\nbranch refs/heads/main\n\n\
            worktree /repo-locked\nHEAD def\ndetached\nlocked reason here\n\n\
            worktree /repo-gone\nHEAD 123\nbranch refs/heads/old\nprunable gitdir file points to non-existent location\n\n\
            worktree /repo.git\nbare\n\n\
            not a block\n\n";

        let listed = parse(output);

        let flags: Vec<_> = listed
            .iter()
            .map(|listed| {
                (
                    listed.path.to_str().unwrap(),
                    listed.bare,
                    listed.locked,
                    listed.prunable,
                )
            })
            .collect();
        assert_eq!(
            flags,
            [
                ("/repo", false, false, false),
                ("/repo-locked", false, true, false),
                ("/repo-gone", false, false, true),
                ("/repo.git", true, false, false),
            ]
        );
    }

    #[test]
    fn a_failed_listing_leaves_the_list_empty() {
        assert_eq!(others(Path::new("/there/is/no/such/folder")).len(), 0);
    }
}
