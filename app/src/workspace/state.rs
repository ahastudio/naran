//! The parts the workspace keeps between frames, grouped by what they are
//! for, so each field says which part of the screen it serves.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui::{Entity, ScrollHandle, SharedString, Subscription, UniformListScrollHandle};
use serde::{Deserialize, Serialize};

use super::detail::{Detail, Inspecting, Marked, Stacked, Viewing};
use super::layout::Divider;
use super::menu::ContextMenu;
use crate::text_input::TextInput;
use crate::watcher::Watching;

/// Where the state of the last run is saved.
pub(super) struct Files {
    pub(super) repositories: PathBuf,
    pub(super) settings: PathBuf,
    pub(super) selection: PathBuf,
    pub(super) layout: PathBuf,
}

impl Files {
    pub(super) fn new(directory: &Path) -> Self {
        Self {
            repositories: directory.join("repositories.json"),
            settings: directory.join("settings.json"),
            selection: directory.join("selection.json"),
            layout: directory.join("layout.json"),
        }
    }
}

/// Whether a group name is being typed for a new group or for a rename.
pub(super) enum EditKind {
    NewGroup,
    RenameGroup(String),
}

impl EditKind {
    /// What the input holds when it opens.
    pub(super) fn initial_text(&self) -> String {
        match self {
            Self::NewGroup => String::new(),
            Self::RenameGroup(name) => name.clone(),
        }
    }
}

/// A name being typed for a new or renamed group.
pub(super) struct Editing {
    pub(super) kind: EditKind,
    pub(super) input: Entity<TextInput>,
    pub(super) _subscription: Subscription,
}

/// The list that arrow keys move through: the one last clicked.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Pane {
    Groups,
    #[default]
    Repositories,
    Commits,
    Files,
}

/// What was pressed last: one of the lists, or the text of the commit
/// information or of the diff. Cmd+A picks from it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum Pressed {
    #[default]
    List,
    Text,
}

/// Scroll handles, so the arrow keys can bring the picked row into view.
#[derive(Default)]
pub(super) struct Scrolls {
    pub(super) history: UniformListScrollHandle,
    pub(super) staged: UniformListScrollHandle,
    pub(super) unstaged: UniformListScrollHandle,
    pub(super) commit_files: UniformListScrollHandle,
    /// The sideways scroll of the diff of a file, so long lines can be read.
    pub(super) diff_sideways: ScrollHandle,
    /// The scroll of the stacked diffs, down and sideways.
    pub(super) stacked: UniformListScrollHandle,
    pub(super) repositories: ScrollHandle,
    pub(super) groups: ScrollHandle,
}

/// What is going on in the background.
#[derive(Default)]
pub(super) struct Progress {
    pub(super) reading: bool,
    /// The system folder picker is open (in another process).
    pub(super) picking_folders: bool,
    /// Go to HEAD is looking for the commit in older pages.
    pub(super) jumping_to_head: bool,
    /// The staging operation in progress, if any.
    pub(super) busy: Option<SharedString>,
}

/// How the last thing the user asked for went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    Succeeded,
    Failed,
}

/// What the user is told.
#[derive(Default)]
pub(super) struct Feedback {
    /// The last result, shown in the status bar, and how it went.
    pub(super) message: Option<(SharedString, Outcome)>,
    /// Folders that could not be added, shown at the top of the repository
    /// column until the user dismisses it.
    pub(super) notice: Option<String>,
}

/// What Naran asks of the system. Tests put stand-ins here.
#[derive(Default)]
pub(super) struct Services {
    /// Shows the system folder picker.
    pub(super) picker: crate::picker::Picker,
    /// Shows folders in Finder.
    pub(super) finder: crate::finder::Finder,
    /// The last folder shown in Finder.
    pub(super) revealed: Option<PathBuf>,
    /// Absent when the platform refused to watch; the app still works
    /// with manual refresh.
    pub(super) watching: Option<Watching>,
}

/// What the detail column shows of the selected repository.
#[derive(Default)]
pub(super) struct Shown {
    pub(super) detail: Option<Detail>,
    /// The changed file whose diff is shown.
    pub(super) viewing: Option<Viewing>,
    /// The files picked with Shift and Cmd in the Staged or Unstaged list.
    pub(super) marked: Marked,
    /// The diffs of the picked files, stacked, while two or more are picked.
    pub(super) stacked: Option<Stacked>,
    /// The commit picked in the commit list, if any.
    pub(super) inspecting: Option<Inspecting>,
}

/// What the pointer and the keys are in the middle of.
#[derive(Default)]
pub(super) struct Interaction {
    /// A group name being typed.
    pub(super) editing: Option<Editing>,
    pub(super) menu: Option<ContextMenu>,
    /// The repositories whose worktrees are shown below them. Not saved:
    /// every window opens with them folded.
    pub(super) open_worktrees: HashSet<PathBuf>,
    pub(super) dragging: Option<Divider>,
    pub(super) pressed: Pressed,
}
