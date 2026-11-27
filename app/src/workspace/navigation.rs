//! Which group and repository are selected, and how the arrow keys and
//! Cmd+A move through the list that was clicked last.

use std::path::{Path, PathBuf};

use gpui::{Context, Window};

use super::Workspace;
use super::detail::Marked;
use super::state::{Pane, Pressed};
use crate::model::{self, Entry};

impl Workspace {
    pub(super) fn group(&self) -> Option<&model::Group> {
        self.selection
            .group
            .as_deref()
            .and_then(|name| self.settings.group(name))
    }

    /// Repositories only appear inside a group, so without one there are none.
    pub(super) fn members(&self) -> Vec<&Entry> {
        match self.group() {
            Some(group) => model::members(&self.entries, group),
            None => Vec::new(),
        }
    }

    fn member_paths(&self) -> Vec<PathBuf> {
        self.members()
            .iter()
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    /// Keeps a group selected whenever one exists.
    pub(super) fn ensure_group(&mut self) {
        let valid = self
            .selection
            .group
            .as_deref()
            .is_some_and(|name| self.settings.group(name).is_some());
        if !valid {
            self.selection.group = self.settings.groups.first().map(|group| group.name.clone());
        }
    }

    pub(super) fn select_group(&mut self, name: Option<String>, cx: &mut Context<Self>) {
        self.switch_group(name);
        self.keep_selection_visible(cx);
        self.save_selection();
        cx.notify();
    }

    fn switch_group(&mut self, name: Option<String>) {
        self.selection.group = name;
        self.ensure_group();
        self.interaction.editing = None;
    }

    /// The selected repository must belong to the selected group. When it
    /// does not, the group's first repository takes its place.
    pub(super) fn keep_selection_visible(&mut self, cx: &mut Context<Self>) {
        let next = self.visible_selection();
        if next != self.selection.repository {
            self.select_repository(next, cx);
        }
    }

    /// Like [`Self::keep_selection_visible`], but reads the details again
    /// even when the selection stays the same.
    pub(super) fn reload_selection(&mut self, cx: &mut Context<Self>) {
        let next = self.visible_selection();
        self.select_repository(next, cx);
    }

    /// The selected repository when the selected group holds it, or else the
    /// group's first one.
    fn visible_selection(&self) -> Option<PathBuf> {
        let members = self.owned_paths();
        let current = self.selection.repository.as_ref();
        let kept = current.filter(|path| members.contains(path)).cloned();
        kept.or_else(|| members.first().cloned())
    }

    pub(super) fn select_repository(&mut self, path: Option<PathBuf>, cx: &mut Context<Self>) {
        if std::mem::replace(&mut self.selection.repository, path.clone()) != path {
            self.forget_repository();
        }
        if let Some(path) = path {
            self.load_detail(path, cx);
        }
        cx.notify();
    }

    /// Another repository is selected: what was shown of the old one goes.
    fn forget_repository(&mut self) {
        self.save_selection();
        self.clear_shown();
    }

    fn clear_shown(&mut self) {
        self.shown.detail = None;
        self.shown.viewing = None;
        self.shown.marked = Marked::default();
        self.shown.stacked = None;
        self.shown.inspecting = None;
    }

    /// Moves through the list that was clicked last.
    pub(super) fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        match self.layout.pane {
            Pane::Groups => self.step_group(delta, cx),
            Pane::Repositories => self.step_selection(delta, cx),
            Pane::Commits => self.step_commit(delta, cx),
            Pane::Files => self.step_file(delta, cx),
        }
    }

    /// Shift with Up or Down grows the picked files of the file lists.
    pub(super) fn extend(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.layout.pane == Pane::Files {
            self.extend_files(delta, cx);
        }
    }

    fn step_group(&mut self, delta: isize, cx: &mut Context<Self>) {
        let names: Vec<String> = self
            .settings
            .groups
            .iter()
            .map(|group| group.name.clone())
            .collect();
        let current = names
            .iter()
            .position(|name| Some(name) == self.selection.group.as_ref());
        if let Some(next) = stepped(names.len(), current, delta) {
            self.select_group(Some(names[next].clone()), cx);
            self.scrolls.groups.scroll_to_item(next + 1); // the title is the first child
        }
    }

    pub(super) fn step_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let members = self.rows_on_screen();
        let paths: Vec<&Path> = members.iter().map(PathBuf::as_path).collect();
        let next =
            model::step(&paths, self.selection.repository.as_deref(), delta).map(Path::to_path_buf);
        self.scroll_to_member(&paths, next.as_deref());
        self.select_repository(next, cx);
    }

    fn scroll_to_member(&self, paths: &[&Path], next: Option<&Path>) {
        let index = next.and_then(|next| paths.iter().position(|path| *path == next));
        // A notice, when there is one, is the first child of the list.
        let first_child = usize::from(self.feedback.notice.is_some());
        if let Some(index) = index {
            self.scrolls
                .repositories
                .scroll_to_item(index + first_child);
        }
    }

    /// Cmd+A: every file of the file list that was pressed last, or else all
    /// the text of the view that was pressed last. In a past commit
    /// it picks the files of the commit, never the text below the list.
    pub(super) fn select_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.layout.pane == Pane::Files && self.interaction.pressed == Pressed::List {
            self.select_all_listed(cx);
        } else {
            self.texts.select_all();
            window.refresh();
        }
    }

    fn select_all_listed(&mut self, cx: &mut Context<Self>) {
        if self.shown.inspecting.is_some() {
            self.select_all_commit_files(cx);
        } else {
            self.select_all_files(self.shown.marked.staging, cx);
        }
    }
}

/// The index one step from `current` in a list of `length` items, kept inside
/// the list. Without a current item the first step lands on an end.
pub(super) fn stepped(length: usize, current: Option<usize>, delta: isize) -> Option<usize> {
    let last = length.checked_sub(1)?;
    Some(current.map_or_else(
        || edge(last, delta),
        |current| current.saturating_add_signed(delta).min(last),
    ))
}

/// Where the first step lands: on the last item going up, otherwise on the first.
fn edge(last: usize, delta: isize) -> usize {
    if delta < 0 { last } else { 0 }
}
