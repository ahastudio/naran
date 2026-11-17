//! What is read from the selected repository: its changes and the first
//! pages of its history with the lanes of the graph.

use std::path::{Path, PathBuf};

use naran_core::log::Commit;
use naran_core::{Change, Error, graph};

use super::lanes::{HeadLink, head_link, head_position};
use crate::model::FileState;

/// Commits read at first and added each time the list is scrolled near its end.
pub(super) const HISTORY_PAGE: usize = 200;

/// What is read from the selected repository, loaded in the background.
pub(in crate::workspace) struct Detail {
    pub(in crate::workspace) path: PathBuf,
    pub(in crate::workspace) changes: Result<Vec<Change>, Error>,
    pub(in crate::workspace) history: Result<Vec<Commit>, Error>,
    /// Graph lanes of each commit in `history`.
    pub(in crate::workspace) rows: Vec<graph::Row>,
    /// A full page came back, so older commits may exist.
    pub(in crate::workspace) more: bool,
    /// An older page is being read.
    pub(in crate::workspace) loading_more: bool,
}

/// The rows of the commit list as the arrow keys step through them.
pub(super) struct CommitRows {
    pub(super) hashes: Vec<String>,
    /// 1 when the Uncommitted changes row comes first.
    pub(super) offset: usize,
}

impl CommitRows {
    pub(super) fn len(&self) -> usize {
        self.hashes.len() + self.offset
    }

    pub(super) fn current(&self, picked: Option<&str>) -> Option<usize> {
        match picked {
            None => (self.offset == 1).then_some(0),
            Some(picked) => self
                .hashes
                .iter()
                .position(|hash| hash == picked)
                .map(|index| index + self.offset),
        }
    }
}

/// Where the jump to HEAD goes.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum HeadJump {
    /// The row of the commit HEAD is on.
    At {
        row: usize,
        hash: String,
    },
    /// Not loaded yet; older commits may hold it.
    Older,
    Missing,
}

impl Detail {
    pub(super) fn read(path: &Path, limit: usize, refs: naran_core::Refs) -> Self {
        let changes = naran_core::changes(path);
        let (more, rows, history) = match naran_core::history(path, limit, refs) {
            Ok(history) => (
                history.commits.len() >= limit,
                history.rows,
                Ok(history.commits),
            ),
            Err(error) => (false, Vec::new(), Err(error)),
        };
        Self {
            path: path.to_path_buf(),
            changes,
            history,
            rows,
            more,
            loading_more: false,
        }
    }

    /// Marks a page as being read, and tells how many commits are listed.
    pub(super) fn begin_page(&mut self) -> Option<usize> {
        let skip = self.history.as_ref().ok()?.len();
        if self.loading_more || !self.more {
            return None;
        }
        self.loading_more = true;
        Some(skip)
    }

    pub(super) fn finish_page(&mut self, skip: usize, page: Result<Vec<Commit>, Error>) {
        self.loading_more = false;
        // A refresh in between changed the list: drop this page.
        if self.history.as_ref().ok().map(Vec::len) == Some(skip) {
            self.append_page(page);
        }
    }

    pub(super) fn append_page(&mut self, page: Result<Vec<Commit>, Error>) {
        self.more = page.as_ref().is_ok_and(|page| page.len() >= HISTORY_PAGE);
        let (Ok(page), Ok(commits)) = (page, self.history.as_mut()) else {
            return;
        };
        commits.extend(page);
        self.rows = graph::layout(commits);
    }

    /// The Uncommitted changes row comes before the commits when there are changes.
    pub(super) fn uncommitted_rows(&self) -> usize {
        usize::from(
            self.changes
                .as_ref()
                .is_ok_and(|changes| !changes.is_empty()),
        )
    }

    pub(super) fn commit_rows(&self) -> Option<CommitRows> {
        let commits = self.history.as_ref().ok()?;
        Some(CommitRows {
            hashes: commits.iter().map(|commit| commit.hash.clone()).collect(),
            offset: self.uncommitted_rows(),
        })
    }

    pub(super) fn head_jump(&self) -> Option<HeadJump> {
        let commits = self.history.as_ref().ok()?;
        Some(match head_position(commits) {
            Some(position) => HeadJump::At {
                row: position + self.uncommitted_rows(),
                hash: commits[position].hash.clone(),
            },
            None if self.more => HeadJump::Older,
            None => HeadJump::Missing,
        })
    }

    pub(super) fn file_state(&self, file: &str) -> Option<FileState> {
        let changes = self.changes.as_ref().ok()?;
        let change = changes.iter().find(|change| change.path == file)?;
        Some(FileState::of(change))
    }

    /// The hash of the first commit, when there are no uncommitted changes.
    pub(super) fn clean_top(&self) -> Option<String> {
        if !self.changes.as_ref().is_ok_and(Vec::is_empty) {
            return None;
        }
        let commits = self.history.as_ref().ok()?;
        commits.first().map(|commit| commit.hash.clone())
    }

    /// Where the Uncommitted changes row joins the graph, when it is there.
    pub(super) fn history_link(&self, commits: &[Commit]) -> Option<HeadLink> {
        let shown = self.uncommitted_rows() > 0;
        shown.then(|| head_link(commits, &self.rows)).flatten()
    }
}
