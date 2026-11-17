//! Reading commits: the history with its graph, a file's history and searches.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Error, git, graph, log};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    pub commits: Vec<log::Commit>,
    pub rows: Vec<graph::Row>,
}

/// Which refs a history starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "saved::Saved", into = "saved::Saved")]
pub struct Refs {
    /// Every ref, including the ones Git keeps for itself, such as the
    /// `refs/original` backup that `git filter-branch` leaves, `refs/stash`
    /// and notes. Otherwise branches, tags and HEAD: what the user means by
    /// the history.
    pub everything: bool,
    /// The remote branches are part of the history.
    pub remotes: bool,
}

impl Refs {
    /// Branches, remote branches, tags and HEAD.
    pub const VISIBLE: Self = Self {
        everything: false,
        remotes: true,
    };
    /// Every ref.
    pub const ALL: Self = Self {
        everything: true,
        remotes: true,
    };

    /// HEAD is left out of a repository without commits, where it names
    /// nothing yet and `git log` would fail on it.
    fn arguments(self, repository: &Path) -> Vec<&'static str> {
        let remotes = (!self.remotes).then_some("--exclude=refs/remotes/*");
        let rest = if self.everything {
            vec!["--all"]
        } else {
            self.visible_arguments(repository)
        };
        remotes.into_iter().chain(rest).collect()
    }

    fn visible_arguments(self, repository: &Path) -> Vec<&'static str> {
        let has_head = git::read(repository, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_ok();
        ["--branches", "--tags"]
            .into_iter()
            .chain(self.remotes.then_some("--remotes"))
            .chain(has_head.then_some("HEAD"))
            .collect()
    }
}

impl Default for Refs {
    fn default() -> Self {
        Self::VISIBLE
    }
}

/// How `layout.json` stores `Refs`: the older files hold the name of one of
/// the two scopes, the newer ones the two flags.
mod saved {
    use serde::{Deserialize, Serialize};

    use super::Refs;

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Saved {
        Scope(Scope),
        Flags { everything: bool, remotes: bool },
    }

    #[derive(Serialize, Deserialize)]
    pub enum Scope {
        Visible,
        All,
    }

    impl From<Saved> for Refs {
        fn from(saved: Saved) -> Self {
            match saved {
                Saved::Scope(Scope::Visible) => Self::VISIBLE,
                Saved::Scope(Scope::All) => Self::ALL,
                Saved::Flags {
                    everything,
                    remotes,
                } => Self {
                    everything,
                    remotes,
                },
            }
        }
    }

    impl From<Refs> for Saved {
        fn from(refs: Refs) -> Self {
            Self::Flags {
                everything: refs.everything,
                remotes: refs.remotes,
            }
        }
    }
}

/// `git log` with the refs to read, followed by `rest`.
fn log_arguments<'a>(repository: &Path, refs: Refs, rest: &[&'a str]) -> Vec<&'a str> {
    let refs = refs.arguments(repository);
    ["log"]
        .into_iter()
        .chain(refs)
        .chain(rest.iter().copied())
        .collect()
}

/// Reads the newest `limit` commits of `refs` and lays out the graph lanes.
pub fn history(repository: &Path, limit: usize, refs: Refs) -> Result<History, Error> {
    let limit = format!("--max-count={limit}");
    let arguments = log_arguments(
        repository,
        refs,
        &["--topo-order", "-z", log::FORMAT, &limit],
    );
    let commits = log::parse(&git::read(repository, &arguments)?);
    let rows = graph::layout(&commits);
    Ok(History { commits, rows })
}

/// One page of the history: `limit` commits after the newest `skip` ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub skip: usize,
    pub limit: usize,
}

/// One page of commits, in the same order as `history`, for loading the
/// list page by page.
pub fn history_page(repository: &Path, page: Page, refs: Refs) -> Result<Vec<log::Commit>, Error> {
    let skip = format!("--skip={}", page.skip);
    let limit = format!("--max-count={}", page.limit);
    let rest = ["--topo-order", "-z", log::FORMAT, &skip, &limit];
    let arguments = log_arguments(repository, refs, &rest);
    Ok(log::parse(&git::read(repository, &arguments)?))
}

/// Commit history of a file, following renames.
pub fn file_history(
    repository: &Path,
    file: &str,
    limit: usize,
) -> Result<Vec<log::Commit>, Error> {
    let limit = format!("--max-count={limit}");
    let output = git::read(
        repository,
        &["log", "--follow", "-z", log::FORMAT, &limit, "--", file],
    )?;
    Ok(log::parse(&output))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchKind {
    Message,
    Author,
    File,
}

impl SearchKind {
    fn arguments(self, query: &str) -> Vec<String> {
        match self {
            Self::Message => vec![format!("--grep={query}")],
            Self::Author => vec![format!("--author={query}")],
            Self::File => vec!["--".to_owned(), query.to_owned()],
        }
    }
}

/// What to look for, and how many commits to return at most.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Search<'a> {
    pub kind: SearchKind,
    pub query: &'a str,
    pub limit: usize,
}

/// Searches the commits of `refs`, ignoring case.
pub fn search(repository: &Path, search: &Search, refs: Refs) -> Result<Vec<log::Commit>, Error> {
    let limit = format!("--max-count={}", search.limit);
    let filter = search.kind.arguments(search.query);
    let rest: Vec<&str> = ["-z", "-i", log::FORMAT, &limit]
        .into_iter()
        .chain(filter.iter().map(String::as_str))
        .collect();
    let arguments = log_arguments(repository, refs, &rest);
    Ok(log::parse(&git::read(repository, &arguments)?))
}
