//! The only changes the column makes to a repository: staging and unstaging
//! files, hunks and lines, each run in the background.

use std::path::{Path, PathBuf};

use gpui::{AsyncApp, Context};
use naran_core::{Error, Staging};

use super::super::Workspace;
use super::super::state::Outcome;
use super::marks::Marked;

async fn run_in_background(
    cx: &AsyncApp,
    path: PathBuf,
    work: impl FnOnce(&Path) -> Result<(), Error> + Send + 'static,
) -> Result<(), Error> {
    let running = cx.background_executor().spawn(async move { work(&path) });
    running.await
}

/// What a link does to lines of the `staging` list: `Stage lines`, or
/// `Unstage hunk` in the Staged list.
pub(super) fn stage_label(staging: Staging, what: &str) -> String {
    let verb = match staging {
        Staging::Staged => "Unstage",
        Staging::Unstaged => "Stage",
    };
    format!("{verb} {what}")
}

impl Workspace {
    /// Dropping files on the other list stages or unstages them.
    pub(in crate::workspace) fn move_files(
        &mut self,
        paths: Vec<String>,
        from: Staging,
        to: Staging,
        cx: &mut Context<Self>,
    ) {
        if from == to || paths.is_empty() {
            return;
        }
        match to {
            Staging::Staged => self.stage(paths, cx),
            Staging::Unstaged => self.unstage(paths, cx),
        }
    }

    pub(super) fn clear_picks(&mut self, cx: &mut Context<Self>) {
        if let Some(viewing) = self.shown.viewing.as_mut() {
            viewing.picked.clear();
        }
        cx.notify();
    }

    /// Runs one staging change on the selected repository in the background,
    /// then reports the result and reads the repository again.
    fn change_repository(
        &mut self,
        label: String,
        work: impl FnOnce(&Path) -> Result<(), Error> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = self.begin_change(&label, cx) else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let result = run_in_background(cx, path.clone(), work).await;
            this.update(cx, |this, cx| this.finish_change(&label, result, path, cx))
                .ok();
        })
        .detach();
    }

    /// The repository to change, unless there is none or another change is running.
    fn begin_change(&mut self, label: &str, cx: &mut Context<Self>) -> Option<PathBuf> {
        let path = self
            .selection
            .repository
            .clone()
            .filter(|_| self.progress.busy.is_none())?;
        self.progress.busy = Some(label.to_string().into());
        cx.notify();
        Some(path)
    }

    fn finish_change(
        &mut self,
        label: &str,
        result: Result<(), Error>,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        self.progress.busy = None;
        match result {
            Ok(()) => self.change_done(label),
            Err(error) => self.change_failed(label, &error),
        }
        Self::refresh_some(vec![path], cx);
    }

    fn change_done(&mut self, label: &str) {
        self.shown.marked = Marked::default();
        self.shown.stacked = None;
        self.feedback.notice = None;
        self.report(format!("{label}: done"), Outcome::Succeeded);
    }

    fn change_failed(&mut self, label: &str, error: &Error) {
        self.report(format!("{label} failed: {error}"), Outcome::Failed);
        self.feedback.notice = Some(format!("{label} failed:\n{error}"));
    }

    pub(in crate::workspace) fn stage(&mut self, files: Vec<String>, cx: &mut Context<Self>) {
        self.change_repository(
            "Stage".to_string(),
            move |path| naran_core::ops::stage(path, &files),
            cx,
        );
    }

    pub(in crate::workspace) fn unstage(&mut self, files: Vec<String>, cx: &mut Context<Self>) {
        self.change_repository(
            "Unstage".to_string(),
            move |path| naran_core::ops::unstage(path, &files),
            cx,
        );
    }

    /// Stages (or unstages, when the diff is the staged one) the given lines
    /// of the open diff.
    pub(in crate::workspace) fn apply_lines(&mut self, lines: Vec<usize>, cx: &mut Context<Self>) {
        let viewing = self.shown.viewing.as_ref();
        let Some(viewing) = viewing.filter(|viewing| !lines.is_empty() && viewing.partial()) else {
            return;
        };
        let (file, staging) = (viewing.file.clone(), viewing.staging);
        self.change_repository(
            stage_label(staging, "lines"),
            move |path| {
                let selection = naran_core::ops::LineSelection {
                    file: &file,
                    staging,
                    lines: &lines,
                };
                naran_core::ops::stage_lines(path, &selection)
            },
            cx,
        );
    }
}
