//! Picking files in the Staged and Unstaged lists: plain, Shift and Cmd
//! clicks, Shift with the arrow keys, and Cmd+A.

use std::collections::BTreeSet;

use gpui::{Context, Modifiers, UniformListScrollHandle};
use naran_core::Staging;

use super::super::Workspace;
use super::super::navigation::stepped;
use super::follow;
use super::marks::{Click, Extension, ListedFile, Marked, file_range};
use crate::model::FileState;

/// The open file and the list it is open in, where a Shift range grows from.
type Cursor = (String, Staging);

impl Workspace {
    /// Files that left their list can no longer be picked.
    pub(super) fn drop_unlisted_marks(&mut self) {
        let listed = self.list_files(self.shown.marked.staging).into_iter();
        let present: BTreeSet<String> = listed.map(|(file, _)| file).collect();
        self.shown
            .marked
            .paths
            .retain(|file| present.contains(file));
    }

    /// The files of one list in the order they are shown.
    pub(super) fn list_files(&self, staging: Staging) -> Vec<(String, FileState)> {
        let Some(Ok(changes)) = self.shown.detail.as_ref().map(|detail| &detail.changes) else {
            return Vec::new();
        };
        changes
            .iter()
            .map(|change| (change.path.clone(), FileState::of(change)))
            .filter(|(_, state)| state.listed_in(staging))
            .collect()
    }

    /// A click on a file row. Plain: that file only. Shift: from the anchor to
    /// this file. Cmd or Ctrl: add or remove this file. The diff of the clicked
    /// file opens.
    pub(in crate::workspace) fn click_file(
        &mut self,
        file: ListedFile,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        let keeps_open = self.mark_by_click(&file.path, file.staging, modifiers);
        self.refresh_stacked(cx);
        if keeps_open {
            self.open_file(file.path, file.staging, file.state, cx);
        } else {
            cx.notify();
        }
    }

    /// Picks files for a click; false when the click took the file off the picks.
    fn mark_by_click(&mut self, path: &str, staging: Staging, modifiers: Modifiers) -> bool {
        match self.shown.marked.click_kind(staging, modifiers) {
            Click::Range => self.mark_range(path, staging),
            Click::Toggle => return self.shown.marked.toggle(path),
            Click::Plain => self.shown.marked = Marked::single(staging, path),
        }
        true
    }

    pub(super) fn mark_range(&mut self, path: &str, staging: Staging) {
        let files = self.list_files(staging);
        let position = |name: &str| files.iter().position(|(file, _)| file == name);
        let ends = self
            .shown
            .marked
            .anchor
            .as_deref()
            .and_then(position)
            .zip(position(path));
        if let Some((from, to)) = ends {
            self.shown.marked.paths = file_range(&files, from, to);
        }
    }

    /// Shift with Up or Down: the picked range grows by one row.
    pub(in crate::workspace) fn extend_files(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.shown.inspecting.is_some() {
            return self.extend_commit_files(delta, cx);
        }
        if let Some(cursor) = self.extension_cursor() {
            self.extend_from(&cursor, delta, cx);
        }
    }

    fn extend_from(&mut self, cursor: &Cursor, delta: isize, cx: &mut Context<Self>) {
        let files = self.list_files(cursor.1);
        let Some(next) = self.mark_extension(&files, cursor, delta) else {
            return;
        };
        let (path, state) = files[next].clone();
        self.open_listed(ListedFile::new(path, cursor.1, state), (next, delta), cx);
    }

    /// The open file and its side.
    fn extension_cursor(&self) -> Option<Cursor> {
        self.shown
            .viewing
            .as_ref()
            .map(|viewing| (viewing.file.clone(), viewing.staging))
    }

    /// Grows the picks from their anchor to the row after the cursor.
    pub(super) fn mark_extension(
        &mut self,
        files: &[(String, FileState)],
        cursor: &Cursor,
        delta: isize,
    ) -> Option<usize> {
        let cursor_row = files.iter().position(|(file, _)| *file == cursor.0)?;
        let next = stepped(files.len(), Some(cursor_row), delta).unwrap_or(cursor_row);
        let extension = Extension {
            staging: cursor.1,
            anchor: self
                .shown
                .marked
                .anchor_in(cursor.1)
                .unwrap_or_else(|| cursor.0.clone()),
            cursor: cursor_row,
            next,
        };
        self.shown.marked = Marked::spanning(files, extension);
        Some(next)
    }

    /// Opens a file of its list, `place.0` rows down, and scrolls the list
    /// to it the way `place.1` moved.
    fn open_listed(&mut self, file: ListedFile, place: (usize, isize), cx: &mut Context<Self>) {
        let (row, delta) = place;
        self.list_scroll(file.staging)
            .scroll_to_item(row, follow(delta));
        self.refresh_stacked(cx);
        self.open_file(file.path, file.staging, file.state, cx);
    }

    /// The scroll handle of the Staged files or Unstaged files list.
    pub(super) fn list_scroll(&self, staging: Staging) -> &UniformListScrollHandle {
        match staging {
            Staging::Staged => &self.scrolls.staged,
            Staging::Unstaged => &self.scrolls.unstaged,
        }
    }

    /// Picks every file of one list (Cmd+A), so they can be dragged across together.
    pub(in crate::workspace) fn select_all_files(
        &mut self,
        staging: Staging,
        cx: &mut Context<Self>,
    ) {
        let files = self.list_files(staging);
        let Some(marked) = Marked::all(staging, &files) else {
            return;
        };
        self.set_marks(marked, cx);
        self.open_when_single(&files, staging, cx);
    }

    pub(super) fn set_marks(&mut self, marked: Marked, cx: &mut Context<Self>) {
        self.shown.marked = marked;
        self.refresh_stacked(cx);
        cx.notify();
    }

    fn open_when_single(
        &mut self,
        files: &[(String, FileState)],
        staging: Staging,
        cx: &mut Context<Self>,
    ) {
        if let [(path, state)] = files {
            self.open_file(path.clone(), staging, *state, cx);
        }
    }
}
