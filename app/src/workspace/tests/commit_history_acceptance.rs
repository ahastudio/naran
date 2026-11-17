//! The commit history as a person uses it: the list and its rows, paging by
//! scrolling, Go to HEAD, the ref buttons and their saved choice, picking a
//! commit and stepping with the arrow keys.

use gpui::{ScrollDelta, ScrollWheelEvent, TouchPhase, point};

use super::*;
use crate::workspace::layout::SavedLayout;

fn head_of(path: &Path) -> String {
    git_output(path, &["rev-parse", "HEAD"]).trim().to_string()
}

fn picked_hash(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Option<String> {
    workspace.read_with(cx, |workspace, _| {
        let picked = workspace.shown.inspecting.as_ref();
        picked.map(|inspecting| inspecting.hash.clone())
    })
}

fn listed(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> usize {
    workspace.read_with(cx, |workspace, _| {
        let detail = workspace.shown.detail.as_ref().unwrap();
        detail.history.as_ref().unwrap().len()
    })
}

fn wheel_down(cx: &mut VisualTestContext, selector: &'static str, distance: f32) {
    let position = cx.debug_bounds(selector).expect(selector).center();
    cx.simulate_event(ScrollWheelEvent {
        position,
        delta: ScrollDelta::Pixels(point(px(0.), px(-distance))),
        modifiers: gpui::Modifiers::default(),
        touch_phase: TouchPhase::Moved,
    });
    cx.run_until_parked();
}

fn choose_beta(workspace: &Entity<Workspace>, fixture: &Fixture, cx: &mut VisualTestContext) {
    let path = fixture.beta.clone();
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(path), cx);
    });
    cx.run_until_parked();
}

#[gpui::test]
fn the_uncommitted_row_comes_first_under_the_column_heads_when_there_are_changes(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    assert!(
        cx.debug_bounds("uncommitted-row").is_none(),
        "alpha is clean"
    );
    choose_beta(&workspace, &fixture, cx);

    let head = cx.debug_bounds("history-head").unwrap();
    let uncommitted = cx.debug_bounds("uncommitted-row").unwrap();
    let first = cx.debug_bounds("commit-row-0").unwrap();
    assert!(head.origin.y < uncommitted.origin.y);
    assert!(uncommitted.origin.y < first.origin.y);
}

#[gpui::test]
fn scrolling_to_the_end_adds_older_commits_and_a_short_history_adds_none(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    add_commits(&fixture.alpha, 230);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(listed(&workspace, cx), 200);

    wheel_down(cx, "commit-row-0", 6000.);
    assert!(listed(&workspace, cx) > 200, "the next page came in");

    choose_beta(&workspace, &fixture, cx);
    wheel_down(cx, "commit-row-0", 6000.);
    assert_eq!(listed(&workspace, cx), 1, "nothing older to load");
}

#[gpui::test]
fn go_to_head_picks_the_head_commit_each_time_it_is_pressed(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    choose_beta(&workspace, &fixture, cx);
    let head = head_of(&fixture.beta);
    assert_eq!(picked_hash(&workspace, cx), None);

    click(cx, "goto-head");
    assert_eq!(picked_hash(&workspace, cx), Some(head.clone()));
    click(cx, "uncommitted-row");
    click(cx, "goto-head");
    assert_eq!(picked_hash(&workspace, cx), Some(head));
}

#[gpui::test]
fn the_go_to_head_button_reads_older_pages_until_head_shows_up(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    add_commits_to(&fixture.alpha, "side", 230);
    let head = head_of(&fixture.alpha);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(listed(&workspace, cx), 200);

    click(cx, "goto-head");
    assert!(listed(&workspace, cx) > 200);
    assert_eq!(picked_hash(&workspace, cx), Some(head));
}

#[gpui::test]
fn the_ref_buttons_save_their_choice_and_a_restart_keeps_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (_workspace, cx) = open(&fixture, cx);
    click(cx, "all-refs");
    click(cx, "remote-branches");
    let saved: SavedLayout = store::load(&fixture.config.join("layout.json"));
    let expected = naran_core::Refs {
        everything: true,
        remotes: false,
    };
    assert_eq!(saved.refs, expected);

    let (again, cx) = open(&fixture, cx);
    again.read_with(cx, |workspace, _| {
        assert_eq!(workspace.layout.refs, expected);
    });
}

#[gpui::test]
fn a_layout_file_in_the_older_form_still_sets_the_refs(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.config.join("layout.json"), r#"{"refs":"All"}"#).unwrap();
    let (workspace, cx) = open(&fixture, cx);
    let refs = |cx: &mut VisualTestContext| workspace.read_with(cx, |w, _| w.layout.refs);
    assert_eq!(refs(cx), naran_core::Refs::ALL);

    click(cx, "all-refs");
    assert_eq!(refs(cx), naran_core::Refs::VISIBLE);
}

#[gpui::test]
fn clicking_a_commit_shows_its_message_and_files_and_the_arrows_step_through_the_list(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    choose_beta(&workspace, &fixture, cx);

    click(cx, "commit-row-0");
    assert!(cx.debug_bounds("commit-info").is_some());
    assert!(cx.debug_bounds("commit-file-README.md").is_some());
    click(cx, "uncommitted-row");
    click(cx, "commits");

    cx.simulate_keystrokes("up");
    assert_eq!(picked_hash(&workspace, cx), None, "the first row stays");
    cx.simulate_keystrokes("down");
    assert_eq!(picked_hash(&workspace, cx), Some(head_of(&fixture.beta)));
    cx.simulate_keystrokes("down");
    assert_eq!(picked_hash(&workspace, cx), Some(head_of(&fixture.beta)));
    cx.simulate_keystrokes("up");
    assert_eq!(picked_hash(&workspace, cx), None);
}

#[gpui::test]
fn a_repository_whose_history_cannot_be_read_shows_the_problem_instead_of_rows(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    assert!(cx.debug_bounds("commit-row-0").is_some());
    assert!(cx.debug_bounds("problem").is_none());

    std::fs::remove_dir_all(fixture.alpha.join(".git")).unwrap();
    workspace.update(cx, Workspace::refresh);
    cx.run_until_parked();

    assert!(cx.debug_bounds("problem").is_some(), "the failure is shown");
    workspace.read_with(cx, |workspace, _| {
        let detail = workspace.shown.detail.as_ref().unwrap();
        assert!(detail.history.is_err(), "so there are no commits to list");
    });
}

#[gpui::test]
fn a_repository_without_commits_has_an_empty_list_and_nothing_to_pick(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let empty = tempfile::tempdir().unwrap();
    git(empty.path(), &["init", "-q", "-b", "main"]);
    let path = empty.path().to_path_buf();
    workspace.update(cx, |workspace, cx| {
        workspace.add_repositories(vec![path.clone()], cx);
        workspace.select_repository(Some(path), cx);
    });
    cx.run_until_parked();

    workspace.read_with(cx, |workspace, _| {
        let detail = workspace.shown.detail.as_ref().unwrap();
        assert_eq!(detail.history.as_ref().unwrap().len(), 0);
        assert!(workspace.shown.inspecting.is_none());
    });
    click(cx, "commits");
    cx.simulate_keystrokes("down");
    assert_eq!(picked_hash(&workspace, cx), None);
}

/// A repository with two commits whose newest is left only by a backup
/// ref that Git keeps for itself, or only by a remote branch.
fn ahead_by_one(fixture: &Fixture, reference: &str) {
    commit(&fixture.alpha, "more.txt", "more\n", "second");
    git(&fixture.alpha, &["update-ref", reference, "HEAD"]);
    git(&fixture.alpha, &["reset", "-q", "--hard", "HEAD~1"]);
}

#[gpui::test]
fn the_all_refs_button_adds_the_commits_only_git_keeps_refs_for_and_a_second_press_removes_them(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    ahead_by_one(&fixture, "refs/original/refs/heads/main");
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(listed(&workspace, cx), 1, "git's own refs are hidden");

    click(cx, "all-refs");
    assert_eq!(listed(&workspace, cx), 2);
    click(cx, "all-refs");
    assert_eq!(listed(&workspace, cx), 1);
}

#[gpui::test]
fn the_remote_branches_button_removes_remote_only_commits_and_a_second_press_brings_them_back(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    ahead_by_one(&fixture, "refs/remotes/origin/main");
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(listed(&workspace, cx), 2, "remote branches are shown");

    click(cx, "remote-branches");
    assert_eq!(listed(&workspace, cx), 1);
    click(cx, "remote-branches");
    assert_eq!(listed(&workspace, cx), 2);
}

#[gpui::test]
fn clicking_a_commit_with_a_body_and_another_committer_reads_both(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("more.txt"), "more").unwrap();
    git(&fixture.alpha, &["add", "."]);
    let status = Command::new("git")
        .args(["commit", "-q", "-m", "A subject", "-m", "The body."])
        .current_dir(&fixture.alpha)
        .env("GIT_COMMITTER_NAME", "Someone Else")
        .env("GIT_COMMITTER_EMAIL", "else@example.com")
        .status()
        .unwrap();
    assert!(status.success());
    let (workspace, cx) = open(&fixture, cx);

    click(cx, "commit-row-0");
    workspace.read_with(cx, |workspace, _| {
        let picked = workspace.shown.inspecting.as_ref().unwrap();
        let detail = picked.detail.as_ref().unwrap().as_ref().unwrap();
        assert!(detail.message.contains("The body."));
        assert_ne!(detail.author, detail.committer);
    });
}
