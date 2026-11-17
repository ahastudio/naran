//! The files picked in the Staged or Unstaged list with Shift and Cmd
//! clicks, and a file as one of those lists shows it.

use std::collections::BTreeSet;

use gpui::Modifiers;
use naran_core::Staging;

use crate::model::FileState;

/// A file as the Staged or Unstaged list shows it: its path, the list, and
/// its state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::workspace) struct ListedFile {
    pub(in crate::workspace) path: String,
    pub(in crate::workspace) staging: Staging,
    pub(in crate::workspace) state: FileState,
}

impl ListedFile {
    pub(in crate::workspace) fn new(
        path: impl Into<String>,
        staging: Staging,
        state: FileState,
    ) -> Self {
        Self {
            path: path.into(),
            staging,
            state,
        }
    }
}

/// The files picked in one of the two lists with Shift and Cmd clicks. Picking
/// stays inside one list; `anchor` is where a Shift range starts.
#[derive(Debug, Clone)]
pub(in crate::workspace) struct Marked {
    pub(in crate::workspace) staging: Staging,
    pub(in crate::workspace) paths: BTreeSet<String>,
    pub(in crate::workspace) anchor: Option<String>,
}

impl Default for Marked {
    /// Nothing picked, in the Unstaged list where changes first show up.
    fn default() -> Self {
        Self {
            staging: Staging::Unstaged,
            paths: BTreeSet::new(),
            anchor: None,
        }
    }
}

/// How a click on a file row changes the picks.
pub(super) enum Click {
    Range,
    Toggle,
    Plain,
}

/// A Shift range grown with the arrow keys: from the anchor to `next`, or
/// from `cursor` when the anchor is not in the list.
pub(super) struct Extension {
    pub(super) staging: Staging,
    pub(super) anchor: String,
    pub(super) cursor: usize,
    pub(super) next: usize,
}

impl Marked {
    pub(super) fn click_kind(&self, staging: Staging, modifiers: Modifiers) -> Click {
        let same_list = self.staging == staging;
        match (
            same_list,
            self.wants_range(modifiers),
            self.wants_toggle(modifiers),
        ) {
            (true, true, _) => Click::Range,
            (true, false, true) => Click::Toggle,
            _ => Click::Plain,
        }
    }

    fn wants_range(&self, modifiers: Modifiers) -> bool {
        modifiers.shift && self.anchor.is_some()
    }

    fn wants_toggle(&self, modifiers: Modifiers) -> bool {
        (modifiers.platform || modifiers.control) && !self.paths.is_empty()
    }

    pub(super) fn single(staging: Staging, path: &str) -> Self {
        Self {
            staging,
            paths: [path.to_string()].into(),
            anchor: Some(path.to_string()),
        }
    }

    /// Every file of a list, anchored at the first; none for an empty list.
    pub(super) fn all(staging: Staging, files: &[(String, FileState)]) -> Option<Self> {
        let (first, _) = files.first()?;
        Some(Self {
            staging,
            paths: files.iter().map(|(file, _)| file.clone()).collect(),
            anchor: Some(first.clone()),
        })
    }

    /// The files of `extension`, from its anchor to its next row.
    pub(super) fn spanning(files: &[(String, FileState)], extension: Extension) -> Self {
        let from = files
            .iter()
            .position(|(file, _)| *file == extension.anchor)
            .unwrap_or(extension.cursor);
        Self {
            staging: extension.staging,
            paths: file_range(files, from, extension.next),
            anchor: Some(extension.anchor),
        }
    }

    /// The anchor of a range in this list, if the picks are of this list.
    pub(super) fn anchor_in(&self, staging: Staging) -> Option<String> {
        self.anchor.clone().filter(|_| self.staging == staging)
    }

    /// Whether `path` of the `staging` list is picked.
    pub(super) fn holds(&self, path: &str, staging: Staging) -> bool {
        self.staging == staging && self.paths.contains(path)
    }

    /// Adds or removes a file; false when it was removed.
    pub(super) fn toggle(&mut self, path: &str) -> bool {
        let added = self.paths.insert(path.to_string());
        if !added {
            self.paths.remove(path);
        }
        self.anchor = Some(path.to_string());
        added
    }
}

pub(super) fn file_range(
    files: &[(String, FileState)],
    from: usize,
    to: usize,
) -> BTreeSet<String> {
    let (low, high) = (from.min(to), from.max(to));
    files[low..=high]
        .iter()
        .map(|(file, _)| file.clone())
        .collect()
}
