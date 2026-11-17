//! What the column does when nothing is loaded yet, or when a read failed:
//! every command waits, and a failure is shown where its part would be.

use super::*;
use crate::theme::Theme;
use crate::workspace::detail::commit::CommitFileKey;
use crate::workspace::detail::stack::{StackRequest, Stacked};

#[gpui::test]
fn a_workspace_with_nothing_loaded_ignores_every_command(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let theme = Theme::dark();

    workspace.update(cx, |workspace, cx| {
        workspace.selection.repository = None;
        workspace.shown.detail = None;
        workspace.shown.inspecting = None;
        workspace.open_file("a.txt".into(), Staging::Unstaged, tracked(), cx);
        workspace.select_all_files(Staging::Unstaged, cx);
        workspace.extend_files(1, cx);
        workspace.step_commit(1, cx);
        workspace.step_working_file(1, cx);
        workspace.jump_to_head(cx);
        workspace.open_commit_file("a.txt".into(), cx);
        workspace.apply_lines(vec![1], cx);
        workspace.stage(vec!["a.txt".into()], cx);
        assert!(workspace.shown.viewing.is_none() && workspace.progress.busy.is_none());
        assert_eq!(workspace.list_files(Staging::Staged), []);
        assert!(workspace.history_rows(0..3, cx).is_empty());
        let rows = workspace.visible_rows(0..3, Staging::Staged, &theme, cx);
        assert!(rows.is_empty());
        assert!(workspace.commit_file_rows(0..3, &theme, cx).is_empty());
        assert!(workspace.stack_rows(0..3, &theme).is_empty());
        let request = StackRequest {
            repository: "/r".into(),
            staging: Staging::Unstaged,
            paths: vec!["a.txt".into()],
        };
        assert!(workspace.stack_jobs(&request).is_empty());
        workspace.add_history_page(Path::new("/other"), 0, Ok(Vec::new()), cx);
    });
}

#[gpui::test]
fn a_failure_is_shown_where_the_missing_part_would_be(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, _| {
        workspace.shown.inspecting = None;
        workspace.shown.detail.as_mut().unwrap().changes = Err(failure("no status"));
    });
    show_problem(&workspace, cx);
    workspace.update(cx, |workspace, _| {
        workspace.shown.detail.as_mut().unwrap().history = Err(failure("no history"));
    });
    show_problem(&workspace, cx);
    workspace.update(cx, |workspace, _| {
        let mut open = viewing(&fixture.alpha, "a.txt");
        open.lines = Some(Err(failure("no diff")));
        workspace.shown.viewing = Some(open);
    });
    show_problem(&workspace, cx);
    workspace.update(cx, |workspace, _| {
        let mut open = viewing(&fixture.alpha, "a.png");
        open.pictures = Some(Err(failure("no picture")));
        workspace.shown.viewing = Some(open);
    });
    show_problem(&workspace, cx);
    workspace.update(cx, |workspace, _| {
        workspace.shown.stacked = Some(Stacked {
            repository: fixture.alpha.clone(),
            staging: Staging::Unstaged,
            paths: Vec::new(),
            rows: Some(Err(failure("no stack"))),
        });
    });
    show_problem(&workspace, cx);
}

fn forget_lines(inspecting: &mut Inspecting) {
    inspecting.lines = None;
}

#[gpui::test]
fn a_picked_commit_that_could_not_be_read_shows_the_failures(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, _| {
        workspace.shown.inspecting = Some(inspecting(&fixture));
    });
    show_problem(&workspace, cx);
    workspace.update(cx, |workspace, _| {
        let mut commit = inspecting(&fixture);
        commit.file = Some("a.png".into());
        commit.pictures = Some(Err(failure("no picture")));
        workspace.shown.inspecting = Some(commit);
    });
    show_problem(&workspace, cx);
    workspace.update(cx, |workspace, _| {
        let mut commit = inspecting(&fixture);
        commit.files = Some(Ok(Vec::new()));
        workspace.shown.inspecting = Some(commit);
    });
    workspace.update(cx, |workspace, cx| {
        workspace.step_file(1, cx);
        let mut key = workspace.shown.inspecting.as_ref().unwrap().key();
        assert!(workspace.with_inspecting(&key, forget_lines, cx));
        assert!(workspace.shown.inspecting.as_ref().unwrap().lines.is_none());
        key.hash = "other".into();
        assert!(!workspace.with_inspecting(&key, forget_lines, cx));
        workspace.shown.inspecting.as_mut().unwrap().files = None;
        workspace.step_file(1, cx);
        workspace.shown.inspecting = None;
        let file = CommitFileKey {
            commit: key,
            file: "a".into(),
        };
        workspace.show_commit_file(&file, cx);
    });
}

#[gpui::test]
fn commands_that_need_a_working_list_or_a_working_commit_wait_for_one(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let theme = Theme::dark();

    workspace.update(cx, |workspace, cx| {
        // A clean repository has no file to step to.
        workspace.shown.inspecting = None;
        workspace.step_file(1, cx);
        assert!(workspace.shown.viewing.is_none());
        // The picked range only grows while no commit is inspected.
        workspace.shown.inspecting = Some(inspecting(&fixture));
        workspace.extend_files(1, cx);
        // A commit whose files are not read shows no rows of them.
        workspace.shown.inspecting.as_mut().unwrap().files = None;
        assert!(workspace.commit_file_rows(0..3, &theme, cx).is_empty());
        // A history that failed to read shows no rows.
        workspace.shown.detail.as_mut().unwrap().history = Err(failure("no history"));
        assert!(workspace.history_rows(0..3, cx).is_empty());
    });
}

#[gpui::test]
fn without_a_selection_or_a_diff_the_commands_change_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    update(&workspace, cx, |workspace, cx| {
        workspace.shown.viewing = None;
        workspace.shown.inspecting = None;
        workspace.shown.detail = None;
        workspace.toggle_pick(0, cx);
        workspace.clear_picks(cx);
        workspace.pick_uncommitted(cx);
        workspace.open_commit_file("README.md".into(), cx);
        workspace.toggle_all_refs(cx);
        assert!(workspace.start_history_page().is_none());
        workspace.selection.repository = None;
        workspace.pick_commit("abc".into(), cx);
    });

    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.shown.inspecting.is_none());
        assert_eq!(workspace.layout.refs, naran_core::Refs::ALL);
    });
}
