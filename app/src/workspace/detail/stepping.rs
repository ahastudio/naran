//! Up and Down in the commit list and in the file lists: the pick moves one
//! row and the list scrolls to keep it in sight.

use gpui::Context;
use naran_core::{Change, Staging};

use super::super::Workspace;
use super::super::navigation::stepped;
use super::follow;
use super::loaded::{CommitRows, Detail};
use super::marks::{ListedFile, Marked};
use crate::model::FileState;

impl Workspace {
    /// Moves the pick one row through the commit list, which starts with the
    /// Uncommitted changes row when there is one.
    pub(in crate::workspace) fn step_commit(&mut self, delta: isize, cx: &mut Context<Self>) {
        if let Some((rows, next)) = self.commit_step(delta) {
            self.pick_row(&rows, next, cx);
            self.scrolls.history.scroll_to_item(next, follow(delta));
        }
    }

    /// The rows of the commit list and the row `delta` away from the pick.
    fn commit_step(&self, delta: isize) -> Option<(CommitRows, usize)> {
        let rows = self.shown.detail.as_ref().and_then(Detail::commit_rows)?;
        let picked = self.shown.inspecting.as_ref();
        let current = rows.current(picked.map(|inspecting| inspecting.hash.as_str()));
        stepped(rows.len(), current, delta).map(|next| (rows, next))
    }

    fn pick_row(&mut self, rows: &CommitRows, next: usize, cx: &mut Context<Self>) {
        match next.checked_sub(rows.offset) {
            Some(index) => self.pick_commit(rows.hashes[index].clone(), cx),
            None => self.pick_uncommitted(cx),
        }
    }

    /// Moves the open file one row through the visible file list: the files
    /// of the picked commit, or Staged files followed by Unstaged files.
    pub(in crate::workspace) fn step_file(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.shown.inspecting.is_some() {
            return self.step_commit_file(delta, cx);
        }
        self.step_working_file(delta, cx);
    }

    fn step_commit_file(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some((next, path)) = self
            .shown
            .inspecting
            .as_mut()
            .and_then(|inspecting| inspecting.step_to(delta))
        else {
            return;
        };
        self.scrolls
            .commit_files
            .scroll_to_item(next, follow(delta));
        self.refresh_stacked(cx);
        self.open_commit_file(path, cx);
    }

    pub(super) fn step_working_file(&mut self, delta: isize, cx: &mut Context<Self>) {
        if let Some((rows, next)) = self.working_step(delta) {
            self.open_working_row(&rows, next, delta, cx);
        }
    }

    /// The rows of both lists and the row `delta` away from the open diff.
    fn working_step(&self, delta: isize) -> Option<(Vec<ListedFile>, usize)> {
        let Some(Ok(changes)) = self.shown.detail.as_ref().map(|detail| &detail.changes) else {
            return None;
        };
        let rows = working_rows(changes);
        let current = self
            .shown
            .viewing
            .as_ref()
            .and_then(|viewing| viewing.row_in(&rows));
        stepped(rows.len(), current, delta).map(|next| (rows, next))
    }

    fn open_working_row(
        &mut self,
        rows: &[ListedFile],
        next: usize,
        delta: isize,
        cx: &mut Context<Self>,
    ) {
        let row = rows[next].clone();
        // The place of the file inside its own list, for the scroll.
        let in_list = rows[..next]
            .iter()
            .filter(|other| other.staging == row.staging)
            .count();
        self.list_scroll(row.staging)
            .scroll_to_item(in_list, follow(delta));
        self.set_marks(Marked::single(row.staging, &row.path), cx);
        self.open_file(row.path, row.staging, row.state, cx);
    }
}

/// The row of a change in the `staging` list, when it is listed there.
fn working_row(change: &Change, staging: Staging) -> Option<ListedFile> {
    let state = FileState::of(change);
    state
        .listed_in(staging)
        .then(|| ListedFile::new(change.path.clone(), staging, state))
}

/// Staged files followed by Unstaged files, as the two lists show them.
fn working_rows(changes: &[Change]) -> Vec<ListedFile> {
    [Staging::Staged, Staging::Unstaged]
        .into_iter()
        .flat_map(|staging| {
            changes
                .iter()
                .filter_map(move |change| working_row(change, staging))
        })
        .collect()
}
