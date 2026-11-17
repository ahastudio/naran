//! The detail column checked part by part, each used the way a person uses
//! it, and what it does when something is missing or failed. The helpers
//! here are shared by the topic modules below.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use gpui::{Context, Modifiers, TestAppContext, VisualTestContext};
use naran_core::log::Commit;
use naran_core::{Change, Error, Staging};

use super::commit::Inspecting;
use super::loaded::Detail;
use super::viewing::Viewing;
use crate::model::{FileKind, FileState};
use crate::workspace::Workspace;
use crate::workspace::tests::{Fixture, fixture, open};

mod commit_picking;
mod commits;
mod diff;
mod files;
mod lanes;
mod missing;
mod stack;

fn failure(message: &str) -> Error {
    Error::InvalidArgument {
        message: message.into(),
    }
}

fn detail(history: Result<Vec<Commit>, Error>) -> Detail {
    Detail {
        path: PathBuf::from("/repository"),
        changes: Err(failure("no status")),
        history,
        rows: Vec::new(),
        more: true,
        loading_more: false,
    }
}

fn tracked() -> FileState {
    FileState {
        kind: FileKind::Tracked,
        staged: false,
        unstaged: true,
    }
}

fn untracked(path: &str) -> Change {
    Change {
        path: path.into(),
        original: None,
        index: '?',
        worktree: '?',
        conflicted: false,
    }
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let place = cx.debug_bounds(selector).expect(selector).center();
    cx.simulate_click(place, Modifiers::default());
    cx.run_until_parked();
}

fn click_at(cx: &mut VisualTestContext, at: gpui::Point<gpui::Pixels>) {
    cx.simulate_click(at, Modifiers::default());
    cx.run_until_parked();
}

fn update(
    workspace: &gpui::Entity<Workspace>,
    cx: &mut VisualTestContext,
    change: impl FnOnce(&mut Workspace, &mut Context<Workspace>),
) {
    workspace.update(cx, change);
    cx.run_until_parked();
}

fn select(workspace: &gpui::Entity<Workspace>, path: &Path, cx: &mut VisualTestContext) {
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(path.to_path_buf()), cx);
    });
    cx.run_until_parked();
}

fn show_problem(workspace: &gpui::Entity<Workspace>, cx: &mut VisualTestContext) {
    workspace.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    assert!(cx.debug_bounds("problem").is_some(), "the failure is shown");
}

fn picked(workspace: &gpui::Entity<Workspace>, cx: &mut VisualTestContext) -> Vec<usize> {
    workspace.read_with(cx, |workspace, _| {
        let viewing = workspace.shown.viewing.as_ref();
        viewing
            .map(|viewing| viewing.picked.iter().copied().collect())
            .unwrap_or_default()
    })
}

/// The unstaged diff of a tracked `file` of `repository`, with nothing read.
fn viewing(repository: &Path, file: &str) -> Viewing {
    Viewing {
        repository: repository.to_path_buf(),
        file: file.into(),
        staging: Staging::Unstaged,
        kind: FileKind::Tracked,
        lines: None,
        pictures: None,
        picked: BTreeSet::new(),
    }
}

/// A picked commit of `alpha` where every read failed.
fn inspecting(fixture: &Fixture) -> Inspecting {
    Inspecting {
        repository: fixture.alpha.clone(),
        hash: "abc".into(),
        files: Some(Err(failure("no files"))),
        detail: Some(Err(failure("no detail"))),
        refs: Vec::new(),
        file: Some("a.txt".into()),
        lines: Some(Err(failure("no lines"))),
        pictures: None,
        picked: BTreeSet::new(),
        anchor: None,
    }
}

/// The index of `+more` among the diff lines of [`changed_file`]: after the
/// four file head lines, the hunk head, `-alpha`, the note that it had no
/// newline at its end, and `+changed`.
const ADDED: usize = 8;

/// `alpha` with a tracked file changed, and its diff open.
fn changed_file(
    cx: &mut TestAppContext,
) -> (Fixture, gpui::Entity<Workspace>, &mut VisualTestContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("README.md"), "changed\nmore\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    update(&workspace, cx, |workspace, cx| {
        workspace.open_file("README.md".into(), Staging::Unstaged, tracked(), cx);
    });
    (fixture, workspace, cx)
}
