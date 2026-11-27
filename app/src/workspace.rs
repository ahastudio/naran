//! The single window: groups on the left, the selected group's repositories
//! in the middle, and the selected repository on the right. Nothing ever
//! replaces the whole screen; selecting only changes a column.
//!
//! The workspace is one GPUI entity whose methods are spread over the child
//! modules by what they are about; each module adds its own `impl Workspace`.

use std::path::{Path, PathBuf};

use gpui::{App, Context, FocusHandle, Focusable, KeyBinding, actions};

use crate::model::{Entry, Selection, Settings};
use crate::selectable::Texts;
use crate::store;

mod chrome;
mod detail;
mod feedback;
mod group_order;
mod groups;
mod layout;
mod membership;
mod menu;
mod navigation;
mod reading;
mod repositories;
mod state;
#[cfg(test)]
mod tests;
mod worktrees;

use layout::SavedLayout;
use state::{Feedback, Files, Interaction, Progress, Scrolls, Services, Shown};

actions!(
    workspace,
    [
        SelectNext,
        SelectPrevious,
        ExtendNext,
        ExtendPrevious,
        Refresh,
        AddRepositories,
        CopyText,
        SelectAllText
    ]
);

const CONTEXT: &str = "Workspace";

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("down", SelectNext, Some(CONTEXT)),
        KeyBinding::new("up", SelectPrevious, Some(CONTEXT)),
        KeyBinding::new("shift-down", ExtendNext, Some(CONTEXT)),
        KeyBinding::new("shift-up", ExtendPrevious, Some(CONTEXT)),
        KeyBinding::new("cmd-r", Refresh, Some(CONTEXT)),
        KeyBinding::new("cmd-o", AddRepositories, Some(CONTEXT)),
        KeyBinding::new("cmd-c", CopyText, Some(CONTEXT)),
        KeyBinding::new("cmd-a", SelectAllText, Some(CONTEXT)),
    ]);
}

pub struct Workspace {
    focus_handle: FocusHandle,
    files: Files,
    paths: Vec<PathBuf>,
    entries: Vec<Entry>,
    settings: Settings,
    selection: Selection,
    shown: Shown,
    scrolls: Scrolls,
    progress: Progress,
    feedback: Feedback,
    services: Services,
    layout: SavedLayout,
    interaction: Interaction,
    /// The text of the commit information and of the diff, which can be
    /// selected with the mouse and copied.
    texts: Texts,
    /// The window title last set: the selected repository, so it is clear
    /// at a glance where you are.
    title: String,
}

impl Focusable for Workspace {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Workspace {
    pub fn new(directory: &Path, cx: &mut Context<Self>) -> Self {
        let mut workspace = Self::load(Files::new(directory), cx.focus_handle());
        workspace.start(cx);
        workspace
    }

    /// The state the last run saved.
    fn load(files: Files, focus_handle: FocusHandle) -> Self {
        let layout: SavedLayout = store::load(&files.layout);
        Self {
            texts: Texts::new(focus_handle.clone()),
            focus_handle,
            paths: store::load(&files.repositories),
            entries: Vec::new(),
            settings: store::load(&files.settings),
            selection: store::load(&files.selection),
            files,
            shown: Shown::default(),
            scrolls: Scrolls::default(),
            progress: Progress::default(),
            feedback: Feedback::default(),
            services: Services::default(),
            layout: layout.clamped(),
            interaction: Interaction::default(),
            title: String::new(),
        }
    }

    /// Tidies the loaded state and starts reading and watching.
    fn start(&mut self, cx: &mut Context<Self>) {
        self.prune();
        self.ensure_group();
        self.start_watching(cx);
        self.refresh(cx);
    }
}
