//! The commit list and a picked commit: pages of history, Go to HEAD, All
//! refs, the message of a commit and the files of it.

use std::process::Command;

use super::*;
use crate::workspace::detail::commit::CommitFileKey;
use crate::workspace::detail::history_view::{all_refs_label, remote_branches_label};
use crate::workspace::detail::loaded::HeadJump;
use crate::workspace::state::Pane;
use crate::workspace::tests::git;

#[test]
fn a_history_that_could_not_be_read_has_no_rows_pages_head_or_top() {
    let mut detail = detail(Err(failure("no history")));

    assert_eq!(detail.begin_page(), None);
    assert!(detail.commit_rows().is_none());
    assert_eq!(detail.head_jump(), None);
    assert_eq!(detail.file_state("README.md"), None);
    detail.changes = Ok(Vec::new());
    assert_eq!(detail.clean_top(), None);
}

#[test]
fn a_page_for_a_list_that_changed_meanwhile_is_dropped() {
    let mut detail = detail(Ok(Vec::new()));
    detail.loading_more = true;

    detail.finish_page(3, Err(failure("no page")));

    assert!(!detail.loading_more);
    assert_eq!(detail.history.as_ref().map(Vec::len), Ok(0));
}

#[test]
fn details_that_cannot_be_read_stay_empty_and_a_failed_page_adds_nothing() {
    let mut detail = Detail::read(
        Path::new("/there/is/no/such/repository"),
        10,
        naran_core::Refs::VISIBLE,
    );
    assert!(detail.changes.is_err() && detail.history.is_err());
    assert!(detail.rows.is_empty() && !detail.more);

    detail.history = Ok(Vec::new());
    detail.append_page(Err(failure("no page")));
    assert!(!detail.more);
    assert_eq!(detail.history.as_ref().unwrap().len(), 0);
    assert_eq!(detail.head_jump(), Some(HeadJump::Missing));
}

#[test]
fn a_commit_whose_files_are_not_read_or_failed_has_no_file_position() {
    let mut inspecting = Inspecting {
        repository: PathBuf::from("/repository"),
        hash: "abc".into(),
        files: None,
        detail: None,
        refs: Vec::new(),
        file: None,
        picked: BTreeSet::new(),
        anchor: None,
        lines: None,
        pictures: None,
    };

    assert!(inspecting.file_position().is_none());
    inspecting.files = Some(Err(failure("no files")));
    assert!(inspecting.file_position().is_none());
}

#[gpui::test]
fn showing_all_refs_reloads_the_list_and_flips_the_label(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let refs =
        |cx: &mut VisualTestContext| workspace.read_with(cx, |workspace, _| workspace.layout.refs);
    assert_eq!(refs(cx), naran_core::Refs::VISIBLE);

    workspace.update(cx, Workspace::toggle_all_refs);
    cx.run_until_parked();

    assert_eq!(refs(cx), naran_core::Refs::ALL);
    assert_eq!(all_refs_label(naran_core::Refs::ALL), "All refs: on");
    workspace.update(cx, Workspace::toggle_all_refs);
    assert_eq!(refs(cx), naran_core::Refs::VISIBLE);
}

#[gpui::test]
fn hiding_remote_branches_reloads_the_list_and_flips_the_label(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let refs =
        |cx: &mut VisualTestContext| workspace.read_with(cx, |workspace, _| workspace.layout.refs);
    assert!(refs(cx).remotes);

    click(cx, "remote-branches");
    cx.run_until_parked();

    assert!(!refs(cx).remotes);
    assert_eq!(remote_branches_label(refs(cx)), "Remote branches: off");
    workspace.update(cx, Workspace::toggle_remote_branches);
    assert_eq!(refs(cx), naran_core::Refs::VISIBLE);
    assert_eq!(remote_branches_label(refs(cx)), "Remote branches: on");
}

#[gpui::test]
fn a_repository_without_commits_has_no_head_to_go_to(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let empty = tempfile::tempdir().unwrap();
    git(empty.path(), &["init", "-q", "-b", "main"]);

    workspace.update(cx, |workspace, cx| {
        workspace.add_repositories(vec![empty.path().to_path_buf()], cx);
    });
    cx.run_until_parked();
    workspace.update(cx, |workspace, cx| {
        workspace.jump_to_head(cx);
        workspace.step_commit(1, cx);
    });

    workspace.read_with(cx, |workspace, _| {
        let message = workspace.feedback.message.clone().unwrap().0;
        assert!(
            message.contains("HEAD is not in the commit list"),
            "{message}"
        );
        assert!(!workspace.progress.jumping_to_head);
    });
}

#[gpui::test]
fn a_commit_with_a_body_and_another_committer_shows_both(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("more.txt"), "more").unwrap();
    git(&fixture.alpha, &["add", "."]);
    let status = Command::new("git")
        .args(["commit", "-q", "-m", "A subject"])
        .args(["-m", "The body of the message."])
        .current_dir(&fixture.alpha)
        .env("GIT_AUTHOR_NAME", "Author")
        .env("GIT_AUTHOR_EMAIL", "author@example.com")
        .env("GIT_COMMITTER_NAME", "Someone Else")
        .env("GIT_COMMITTER_EMAIL", "else@example.com")
        .status()
        .unwrap();
    assert!(status.success());
    let (workspace, cx) = open(&fixture, cx);

    let hash = workspace.read_with(cx, |workspace, _| {
        let detail = workspace.shown.detail.as_ref().unwrap();
        detail.history.as_ref().unwrap()[0].hash.clone()
    });
    workspace.update(cx, |workspace, cx| workspace.pick_commit(hash, cx));
    cx.run_until_parked();

    assert!(cx.debug_bounds("commit-info").is_some());
    workspace.read_with(cx, |workspace, _| {
        let inspecting = workspace.shown.inspecting.as_ref().unwrap();
        let detail = inspecting.detail.as_ref().unwrap().as_ref().unwrap();
        assert!(detail.message.contains("The body of the message."));
        assert_ne!(detail.author, detail.committer);
    });
}

#[gpui::test]
fn the_commit_list_rows_and_buttons_pick_and_move(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let inspecting = |workspace: &gpui::Entity<Workspace>, cx: &mut VisualTestContext| {
        workspace.read_with(cx, |workspace, _| workspace.shown.inspecting.is_some())
    };
    select(&workspace, &fixture.beta, cx);
    assert!(!inspecting(&workspace, cx), "beta has uncommitted changes");

    click(cx, "commit-row-0");
    assert!(inspecting(&workspace, cx));
    click(cx, "commit-file-README.md");
    workspace.read_with(cx, |workspace, _| {
        let inspecting = workspace.shown.inspecting.as_ref();
        let file = inspecting.and_then(|inspecting| inspecting.file.clone());
        assert_eq!(file.as_deref(), Some("README.md"));
    });
    click(cx, "uncommitted-row");
    assert!(!inspecting(&workspace, cx));
    click(cx, "commits");
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.layout.pane, Pane::Commits);
    });
    click(cx, "all-refs");
    click(cx, "goto-head");
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.layout.refs, naran_core::Refs::ALL);
    });
}

#[gpui::test]
fn a_result_for_another_file_of_the_commit_is_dropped(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    click(cx, "commit-file-README.md");
    let forget_lines = |inspecting: &mut Inspecting| inspecting.lines = None;
    let lines = |workspace: &Workspace| {
        let inspecting = workspace.shown.inspecting.as_ref();
        inspecting
            .and_then(|inspecting| inspecting.lines.as_ref())
            .is_some()
    };

    update(&workspace, cx, |workspace, cx| {
        let commit = workspace.shown.inspecting.as_ref().unwrap().key();
        let other = CommitFileKey {
            commit: commit.clone(),
            file: "other.txt".into(),
        };
        workspace.with_inspected_file(&other, forget_lines, cx);
        assert!(lines(workspace), "another file's result is dropped");
        let open = CommitFileKey {
            commit,
            file: "README.md".into(),
        };
        workspace.with_inspected_file(&open, forget_lines, cx);
        assert!(!lines(workspace));
    });
}
