//! Worktrees below their repository: which repositories are open, the rows
//! on screen, and the repository a worktree belongs to.

use std::path::{Path, PathBuf};

use gpui::{Context, Div, Stateful, div, prelude::*};

use super::Workspace;
use crate::model::Entry;
use crate::theme::Theme;
use crate::watcher::Watching;
use crate::widgets::{Fold, worktree_banner, worktree_row, worktree_slot};

impl Workspace {
    pub(super) fn is_open(&self, path: &Path) -> bool {
        self.interaction.open_worktrees.contains(path)
    }

    /// The paths of the rows on screen: each repository of the group, and
    /// below it the worktrees when it is open.
    pub(super) fn rows_on_screen(&self) -> Vec<PathBuf> {
        self.listed(|entry| self.is_open(entry.path()))
    }

    /// The paths that belong to the group: its repositories and all of their
    /// worktrees, shown or not.
    pub(super) fn owned_paths(&self) -> Vec<PathBuf> {
        self.listed(|_| true)
    }

    /// The repositories with the worktrees of those `shown` accepts.
    fn listed(&self, shown: impl Fn(&Entry) -> bool) -> Vec<PathBuf> {
        let rows = |entry: &&Entry| {
            let worktrees = if shown(entry) {
                entry.worktrees()
            } else {
                Vec::new()
            };
            let below = worktrees
                .iter()
                .map(|worktree| worktree.path().to_path_buf());
            [entry.path().to_path_buf()]
                .into_iter()
                .chain(below)
                .collect::<Vec<_>>()
        };
        self.members().iter().flat_map(rows).collect()
    }

    /// The repository the path belongs to: itself, or the one that lists it
    /// as a worktree. None for a path nobody lists.
    pub(super) fn owner_of(&self, path: &Path) -> Option<PathBuf> {
        let owns = |entry: &&Entry| {
            entry.path() == path || entry.worktrees().iter().any(|tree| tree.path() == path)
        };
        let owner = self.entries.iter().find(owns);
        owner.map(|entry| entry.path().to_path_buf())
    }

    /// Opens the worktrees below a repository, or folds them. Folding with
    /// one of them selected selects the repository, so the selection never
    /// hides.
    pub(super) fn toggle_worktrees(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if self.interaction.open_worktrees.remove(&path) {
            self.leave_folded(&path, cx);
        } else {
            self.interaction.open_worktrees.insert(path);
        }
        cx.notify();
    }

    fn leave_folded(&mut self, path: &Path, cx: &mut Context<Self>) {
        let selected = self.selection.repository.clone();
        let owner = selected
            .as_deref()
            .and_then(|selected| self.owner_of(selected));
        if selected.as_deref() != Some(path) && owner.as_deref() == Some(path) {
            self.select_repository(Some(path.to_path_buf()), cx);
        }
    }

    /// The bar above the detail when the selection is a worktree: which
    /// repository it belongs to, its folder and its branch.
    pub(super) fn worktree_banner(&self, theme: &Theme) -> Option<Stateful<Div>> {
        let path = self.selection.repository.as_deref()?;
        let (owner, worktree) = self.entries.iter().find_map(|entry| {
            let found = entry.worktrees().into_iter().find(|w| w.path() == path);
            found.map(|worktree| (entry, worktree))
        })?;
        Some(worktree_banner(&owner.folder(), &worktree, theme))
    }

    /// Watches the folder of every worktree for its repository, so a change
    /// there reads the repository again, and lets go of the folders of
    /// worktrees that are gone.
    pub(super) fn watch_worktrees(&mut self) {
        let pairs = self.linked_folders();
        if let Some(watching) = self.services.watching.as_mut() {
            sync_watches(watching, &pairs);
        }
    }

    /// Each worktree folder with the repository that lists it.
    fn linked_folders(&self) -> Vec<(PathBuf, PathBuf)> {
        let linked = |entry: &Entry| {
            let trees = entry.worktrees();
            let owner = entry.path();
            let pairs = trees
                .iter()
                .map(|tree| (tree.path().to_path_buf(), owner.to_path_buf()));
            pairs.collect::<Vec<_>>()
        };
        self.entries.iter().flat_map(linked).collect()
    }

    /// The toggle at the right end of a repository row, for a repository
    /// that has other worktrees.
    pub(super) fn worktree_toggle(
        &self,
        entry: &Entry,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        let toggle = worktree_slot(entry, self.fold_of(entry.path()), theme);
        (entry.worktree_count() > 0).then(|| open_on_click(toggle, entry.path(), cx))
    }

    fn fold_of(&self, path: &Path) -> Fold {
        if self.is_open(path) {
            Fold::Open
        } else {
            Fold::Folded
        }
    }

    /// The rows below an open repository.
    pub(super) fn worktree_rows(
        &self,
        entry: &Entry,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Vec<Div> {
        if !self.is_open(entry.path()) {
            return Vec::new();
        }
        let rows = entry.worktrees();
        rows.iter()
            .map(|worktree| self.worktree_item(worktree, theme, cx))
            .collect()
    }

    fn worktree_item(&self, worktree: &Entry, theme: &Theme, cx: &mut Context<Self>) -> Div {
        let path = worktree.path().to_path_buf();
        let selected = self.selection.repository.as_deref() == Some(worktree.path());
        let row = worktree_row(worktree, theme).on_click(cx.listener(move |this, _, _, cx| {
            this.select_repository(Some(path.clone()), cx);
        }));
        div()
            .border_b_1()
            .border_color(theme.border)
            .when(selected, |frame| frame.bg(theme.selected))
            .child(row)
    }
}

/// Watches the worktree folders listed and lets go of the others.
fn sync_watches(watching: &mut Watching, pairs: &[(PathBuf, PathBuf)]) {
    let folders: Vec<_> = pairs.iter().map(|(folder, _)| folder.clone()).collect();
    watching.keep_linked(&folders);
    for (folder, owner) in pairs {
        watching.watch_linked(folder, owner);
    }
}

fn open_on_click(toggle: Stateful<Div>, path: &Path, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let path = path.to_path_buf();
    toggle.on_click(cx.listener(move |this, _, _, cx| {
        this.toggle_worktrees(path.clone(), cx);
    }))
}
