//! Reading the repositories off the UI thread, watching them for changes,
//! and forgetting the ones no group holds any more.

use std::path::PathBuf;

use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui::{AsyncApp, Context, WeakEntity};

use super::Workspace;
use super::state::Outcome;
use crate::model::Entry;
use crate::watcher::Watching;

/// How many `git` processes run at once when reading every repository.
const READ_CONCURRENCY: usize = 8;

impl Workspace {
    /// Reads every repository again, off the UI thread.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let paths = self.begin_reading(cx);
        cx.spawn(async move |this, cx| {
            let entries = read_in_background(cx, paths).await;
            this.update(cx, |this, cx| this.finish_refresh(entries, cx))
                .ok();
        })
        .detach();
    }

    fn begin_reading(&mut self, cx: &mut Context<Self>) -> Vec<PathBuf> {
        self.progress.reading = true;
        cx.notify();
        self.paths.clone()
    }

    fn finish_refresh(&mut self, entries: Vec<Entry>, cx: &mut Context<Self>) {
        self.entries = entries;
        self.progress.reading = false;
        self.reload_selection(cx);
        cx.notify();
    }

    /// Reads the given repositories again and replaces only their entries.
    /// Details reload only when the selected repository is among them.
    pub(super) fn refresh_some(paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let entries = read_in_background(cx, paths.clone()).await;
            this.update(cx, |this, cx| this.merge_entries(&paths, entries, cx))
                .ok();
        })
        .detach();
    }

    fn merge_entries(&mut self, paths: &[PathBuf], entries: Vec<Entry>, cx: &mut Context<Self>) {
        let changed = self.replace_entries(entries);
        if self.is_selected_among(paths) {
            self.reload_selection(cx);
        }
        if changed {
            cx.notify();
        }
    }

    fn is_selected_among(&self, paths: &[PathBuf]) -> bool {
        let selected = self.selection.repository.as_ref();
        selected.is_some_and(|path| paths.contains(path))
    }

    /// Replaces the entries that differ from the ones read; tells whether any did.
    fn replace_entries(&mut self, entries: Vec<Entry>) -> bool {
        let replace = |changed, entry| self.replace_entry(entry) || changed;
        entries.into_iter().fold(false, replace)
    }

    /// Puts the entry in place of the one for the same path. True when that
    /// changed something.
    pub(super) fn replace_entry(&mut self, entry: Entry) -> bool {
        let changed =
            |existing: &&mut Entry| existing.path() == entry.path() && **existing != entry;
        let Some(existing) = self.entries.iter_mut().find(changed) else {
            return false;
        };
        *existing = entry;
        true
    }

    /// Starts watching every repository. Changes found by the watcher are
    /// read again in the background.
    pub(super) fn start_watching(&mut self, cx: &mut Context<Self>) {
        let (sender, receiver) = futures::channel::mpsc::unbounded();
        self.services.watching = self.watch_all(Watching::start(sender));
        cx.spawn(async move |this, cx| forward_changes(receiver, this, cx).await)
            .detach();
    }

    pub(super) fn watch_all(&mut self, started: notify::Result<Watching>) -> Option<Watching> {
        let Ok(mut watching) = started else {
            self.report(
                "Could not watch the repositories for changes. Use Refresh.",
                Outcome::Failed,
            );
            return None;
        };
        self.paths.iter().for_each(|path| watching.watch(path));
        Some(watching)
    }

    /// Drops repositories that no group holds any more. They could never be
    /// seen again, so they should not be read or watched either.
    pub(super) fn prune(&mut self) {
        let orphans = self.orphans();
        if orphans.is_empty() {
            return;
        }
        self.forget_paths(&orphans);
        self.save_paths();
    }

    fn orphans(&self) -> Vec<PathBuf> {
        let orphans = self
            .paths
            .iter()
            .filter(|path| !self.settings.in_any_group(path));
        orphans.cloned().collect()
    }

    fn forget_paths(&mut self, orphans: &[PathBuf]) {
        if let Some(watching) = self.services.watching.as_mut() {
            for path in orphans {
                watching.unwatch(path);
            }
        }
        self.paths.retain(|path| !orphans.contains(path));
        self.entries
            .retain(|entry| !orphans.iter().any(|path| path == entry.path()));
    }
}

fn read_all(paths: &[PathBuf], limit: usize) -> Vec<Entry> {
    naran_core::batch::parallel(paths, limit, |path| Entry::read(path))
        .into_iter()
        .map(|(_, entry)| entry)
        .collect()
}

pub(super) async fn read_in_background(cx: &AsyncApp, paths: Vec<PathBuf>) -> Vec<Entry> {
    let reading = cx
        .background_executor()
        .spawn(async move { read_all(&paths, READ_CONCURRENCY) });
    reading.await
}

/// Reads again what the watcher reports, until the window is gone.
pub(super) async fn forward_changes(
    mut receiver: UnboundedReceiver<Vec<PathBuf>>,
    this: WeakEntity<Workspace>,
    cx: &mut AsyncApp,
) {
    while let Some(paths) = receiver.next().await {
        let refreshing = this.update(cx, |_, cx| Workspace::refresh_some(paths, cx));
        if refreshing.is_err() {
            break;
        }
    }
}
