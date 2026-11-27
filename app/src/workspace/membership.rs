//! Putting repositories into the selected group, moving them inside it,
//! and taking them out.

use std::path::{Path, PathBuf};

use gpui::{Context, PromptLevel, Window};

use super::Workspace;
use super::chrome::folder_name;
use super::navigation::neighbour;
use super::reading::read_in_background;
use super::state::Outcome;
use crate::model::Entry;
use crate::picker::PickOutcome;

/// What adding a batch of repositories did.
#[derive(Default)]
struct Addition {
    added: usize,
    /// One line for each folder that could not be read.
    failures: Vec<String>,
    /// The last folder that was put into the group.
    last: Option<PathBuf>,
}

impl Workspace {
    /// Opens the system folder picker in another process (see `picker.rs`), so
    /// the window keeps working while it opens. Cmd+O does the same.
    pub(super) fn prompt_for_repositories(&mut self, cx: &mut Context<Self>) {
        if self.group().is_none() || self.progress.picking_folders {
            return;
        }
        self.progress.picking_folders = true;
        cx.notify();
        self.pick_in_background(cx);
    }

    fn pick_in_background(&self, cx: &mut Context<Self>) {
        let picker = self.services.picker.clone();
        cx.spawn(async move |this, cx| {
            let outcome = cx
                .background_executor()
                .spawn(async move { picker.choose() })
                .await;
            this.update(cx, |this, cx| this.finish_picking(outcome, cx))
                .ok();
        })
        .detach();
    }

    /// The picker answered: add what was chosen, say so when it failed, and do
    /// nothing when the user closed it.
    pub(super) fn finish_picking(&mut self, outcome: PickOutcome, cx: &mut Context<Self>) {
        self.progress.picking_folders = false;
        match outcome {
            PickOutcome::Picked(paths) => self.add_repositories(paths, cx),
            PickOutcome::Canceled => cx.notify(),
            PickOutcome::Failed(why) => self.pick_failed(&why, cx),
        }
    }

    fn pick_failed(&mut self, why: &str, cx: &mut Context<Self>) {
        let hint = "You can drop folders onto the window instead.";
        self.report(
            format!("The folder picker failed: {why}. {hint}"),
            Outcome::Failed,
        );
        cx.notify();
    }

    /// Checks each folder in the background and puts the readable ones into
    /// the selected group. Without a group there is nowhere to put them.
    pub(super) fn add_repositories(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let Some(group) = self.selection.group.clone() else {
            self.report(
                "Create a group first. Repositories live inside groups.",
                Outcome::Failed,
            );
            cx.notify();
            return;
        };
        cx.spawn(async move |this, cx| {
            let entries = read_in_background(cx, paths).await;
            this.update(cx, |this, cx| this.finish_adding(entries, &group, cx))
                .ok();
        })
        .detach();
    }

    /// Puts the readable entries into the group and says how it went.
    fn finish_adding(&mut self, entries: Vec<Entry>, group: &str, cx: &mut Context<Self>) {
        let addition = self.add_entries(entries, group);
        self.save_membership();
        self.report_addition(&addition);
        addition
            .last
            .into_iter()
            .for_each(|last| self.select_repository(Some(last), cx));
        cx.notify();
    }

    fn add_entries(&mut self, entries: Vec<Entry>, group: &str) -> Addition {
        let mut addition = Addition::default();
        for entry in entries {
            self.add_entry(entry, group, &mut addition);
        }
        addition
    }

    fn save_membership(&mut self) {
        self.save_paths();
        self.save_settings();
    }

    fn add_entry(&mut self, entry: Entry, group: &str, addition: &mut Addition) {
        match &entry {
            Entry::Failed { error, .. } => addition
                .failures
                .push(format!("{}: {error}", entry.path().display())),
            Entry::Ready(_) => self.keep_entry(entry, group, addition),
        }
    }

    fn keep_entry(&mut self, entry: Entry, group: &str, addition: &mut Addition) {
        let path = entry.path().to_path_buf();
        addition.added += usize::from(self.register(entry, &path));
        self.settings.add_to_group(group, &path);
        addition.last = Some(path);
    }

    /// Reads and watches a repository the app did not know yet. False when it
    /// was already known.
    fn register(&mut self, entry: Entry, path: &Path) -> bool {
        if self.paths.iter().any(|known| known == path) {
            return false;
        }
        self.learn(entry, path);
        true
    }

    fn learn(&mut self, entry: Entry, path: &Path) {
        self.paths.push(path.to_path_buf());
        self.services
            .watching
            .iter_mut()
            .for_each(|watching| watching.watch(path));
        self.entries.push(entry);
    }

    fn report_addition(&mut self, addition: &Addition) {
        match addition.failures.first() {
            Some(first) => self.report(
                format!(
                    "Added {}, failed {}: {first}",
                    addition.added,
                    addition.failures.len()
                ),
                Outcome::Failed,
            ),
            None => self.report_all_added(addition.added),
        }
        if !addition.failures.is_empty() {
            self.feedback.notice = Some(format!("Not added:\n{}", addition.failures.join("\n")));
        }
    }

    fn report_all_added(&mut self, added: usize) {
        self.report(format!("Added {added} repositories."), Outcome::Succeeded);
        self.feedback.notice = None;
    }

    /// Puts a dragged repository into the slot of the row it was dropped on,
    /// inside the selected group, and keeps that order.
    pub(super) fn move_repository(&mut self, path: &Path, target: &Path, cx: &mut Context<Self>) {
        let Some(group) = self.selection.group.clone() else {
            return;
        };
        if self.settings.move_in_group(&group, path, target) {
            self.save_settings();
            cx.notify();
        }
    }

    /// Moves a dragged repository from the group it was dragged from to the
    /// end of another group. The folder stays where it is, and the selection
    /// moves on when the selected repository is the one that left.
    pub(super) fn move_to_group(
        &mut self,
        path: &Path,
        from: &str,
        to: &str,
        cx: &mut Context<Self>,
    ) {
        let next = neighbour(&self.member_paths(), path);
        if self.settings.move_to_group(from, path, to) {
            self.save_settings();
            self.leave_selection(path, next, cx);
        }
    }

    /// Selects `next` when `path`, which left the group, was selected.
    fn leave_selection(&mut self, path: &Path, next: Option<PathBuf>, cx: &mut Context<Self>) {
        let selected = self.selection.repository.as_deref();
        if selected.is_some_and(|selected| self.owner_of(selected).as_deref() == Some(path)) {
            self.select_repository(next, cx);
        }
        cx.notify();
    }

    /// Shows the folder in Finder.
    pub(super) fn reveal(&mut self, path: &Path) {
        self.services.revealed = Some(path.to_path_buf());
        if let Err(error) = self.services.finder.reveal(path) {
            self.report(
                format!("Could not show the folder in Finder: {error}"),
                Outcome::Failed,
            );
        }
    }

    /// Asks first. Nothing is removed unless the answer is Remove.
    pub(super) fn confirm_remove(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(group) = self.selection.group.clone() else {
            return;
        };
        let question = format!(
            "Remove \"{}\" from the group \"{group}\"?",
            folder_name(&path)
        );
        let answer = window.prompt(
            PromptLevel::Warning,
            &question,
            Some("A repository left in no group leaves Naran. Its folder is not touched."),
            &["Remove", "Cancel"],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await == Ok(0) {
                this.update(cx, |this, cx| this.remove(&path, cx)).ok();
            }
        })
        .detach();
    }

    /// Takes the repository out of the selected group. A repository left in
    /// no group leaves Naran; its folder is untouched.
    pub(super) fn remove(&mut self, path: &Path, cx: &mut Context<Self>) {
        let Some(name) = self.selection.group.clone() else {
            return;
        };
        self.take_out_of_group(&name, path);
        self.keep_selection_visible(cx);
        cx.notify();
    }

    fn take_out_of_group(&mut self, name: &str, path: &Path) {
        self.settings.remove_from_group(name, path);
        self.save_settings();
        self.prune();
    }
}
