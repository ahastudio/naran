//! Staging as a person does it: real clicks, keys and drags on the two lists
//! and on the diff.

use gpui::{Modifiers, MouseButton};

use super::*;

/// Selector names are static; tests leak the few they build.
pub(super) fn named(id: &str) -> &'static str {
    Box::leak(id.to_owned().into_boxed_str())
}

pub(super) fn center(cx: &mut VisualTestContext, id: &str) -> gpui::Point<Pixels> {
    cx.debug_bounds(named(id))
        .unwrap_or_else(|| panic!("{id} is on screen"))
        .center()
}

/// Presses on a row, moves in steps and lets go on a list.
pub(super) fn drag(cx: &mut VisualTestContext, row: &str, list: &str) {
    let (from, to) = (center(cx, row), center(cx, list));
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
    for step in 1..=8_u8 {
        cx.simulate_mouse_move(
            from + (to - from) * (f32::from(step) / 8.),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();
    }
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

pub(super) fn click_row(cx: &mut VisualTestContext, id: &str, modifiers: Modifiers) {
    let at = center(cx, id);
    cx.simulate_click(at, modifiers);
    cx.run_until_parked();
}

pub(super) fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::none()
    }
}

pub(super) fn cmd() -> Modifiers {
    Modifiers {
        platform: true,
        ..Modifiers::none()
    }
}

pub(super) fn listed(cx: &mut VisualTestContext, id: &str) -> bool {
    cx.debug_bounds(named(id)).is_some()
}

pub(super) fn open_beta<'a>(
    fixture: &Fixture,
    cx: &'a mut TestAppContext,
) -> (Entity<Workspace>, &'a mut VisualTestContext) {
    let (workspace, cx) = open(fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(fixture.beta.clone()), cx);
    });
    cx.run_until_parked();
    (workspace, cx)
}

pub(super) fn conflict(fixture: &Fixture) {
    let path = &fixture.alpha;
    for branch in ["topic", "main"] {
        git(path, &["switch", "-q", "-C", branch]);
        commit(path, "README.md", branch, branch);
        git(path, &["switch", "-q", "main"]);
    }
    let merge = Command::new("git")
        .args(["merge", "-q", "topic"])
        .current_dir(path)
        .output();
    assert!(!merge.unwrap().status.success());
}

#[gpui::test]
fn dragging_a_row_stages_and_unstages_it_and_the_same_list_does_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open_beta(&fixture, cx);

    drag(cx, "file-unstaged-draft.txt", "list-unstaged");
    let ran = workspace.read_with(cx, |workspace, _| workspace.feedback.message.is_some());
    assert!(!ran, "a drop on the same list runs no change");
    assert_eq!(
        git_output(&fixture.beta, &["status", "--short"]),
        "?? draft.txt\n"
    );

    drag(cx, "file-unstaged-draft.txt", "list-staged");
    assert!(listed(cx, "file-staged-draft.txt"));
    assert_eq!(
        git_output(&fixture.beta, &["status", "--short"]),
        "A  draft.txt\n"
    );

    drag(cx, "file-staged-draft.txt", "list-unstaged");
    assert!(listed(cx, "file-unstaged-draft.txt"));
    assert_eq!(
        git_output(&fixture.beta, &["status", "--short"]),
        "?? draft.txt\n"
    );
}

#[gpui::test]
fn unstaging_works_before_the_first_commit(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    git(&fixture.beta, &["switch", "-q", "--orphan", "fresh"]);
    std::fs::write(fixture.beta.join("new.txt"), "new\n").unwrap();
    let (_workspace, cx) = open_beta(&fixture, cx);

    drag(cx, "file-unstaged-new.txt", "list-staged");
    assert!(listed(cx, "file-staged-new.txt"));
    drag(cx, "file-staged-new.txt", "list-unstaged");
    assert!(listed(cx, "file-unstaged-new.txt"));
    assert_eq!(
        git_output(&fixture.beta, &["status", "--short"]),
        "?? draft.txt\n?? new.txt\n"
    );
}

#[gpui::test]
fn clicks_and_keys_pick_files_and_dragging_one_takes_them_all(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);

    click_row(cx, "file-unstaged-a.txt", Modifiers::none());
    assert_eq!(picked(&workspace, cx), ["a.txt"]);
    click_row(cx, "file-unstaged-c.txt", shift());
    assert_eq!(picked(&workspace, cx), ["a.txt", "b.txt", "c.txt"]);
    click_row(cx, "file-unstaged-b.txt", cmd());
    assert_eq!(picked(&workspace, cx), ["a.txt", "c.txt"]);
    click_row(cx, "file-unstaged-b.txt", cmd());
    assert_eq!(picked(&workspace, cx), ["a.txt", "b.txt", "c.txt"]);
    click_row(cx, "file-unstaged-a.txt", Modifiers::none());
    click_row(cx, "file-unstaged-c.txt", shift());

    cx.simulate_keystrokes("shift-down");
    cx.run_until_parked();
    assert_eq!(picked(&workspace, cx), ["a.txt", "b.txt", "c.txt", "d.txt"]);
    cx.simulate_keystrokes("shift-up");
    cx.run_until_parked();
    assert_eq!(picked(&workspace, cx), ["a.txt", "b.txt", "c.txt"]);

    drag(cx, "file-unstaged-b.txt", "list-staged");
    for name in ["a.txt", "b.txt", "c.txt"] {
        assert!(listed(cx, &format!("file-staged-{name}")), "{name}");
    }
    assert!(listed(cx, "file-unstaged-d.txt"));
    assert_eq!(picked(&workspace, cx), [] as [String; 0]);
}

#[gpui::test]
fn cmd_a_picks_a_single_file_list_and_does_nothing_on_an_empty_one(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open_beta(&fixture, cx);
    click_row(cx, "file-unstaged-draft.txt", Modifiers::none());

    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();
    assert_eq!(picked(&workspace, cx), ["draft.txt"]);

    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(fixture.gamma.clone()), cx);
        workspace.layout.pane = Pane::Files;
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();
    assert_eq!(picked(&workspace, cx), [] as [String; 0]);
}

#[gpui::test]
fn a_pick_that_leaves_the_list_is_dropped_when_the_repository_is_read_again(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);
    click_row(cx, "file-unstaged-a.txt", Modifiers::none());
    click_row(cx, "file-unstaged-c.txt", shift());
    assert_eq!(picked(&workspace, cx), ["a.txt", "b.txt", "c.txt"]);

    std::fs::remove_file(fixture.alpha.join("b.txt")).unwrap();
    cx.simulate_keystrokes("cmd-r");
    cx.run_until_parked();
    assert_eq!(picked(&workspace, cx), ["a.txt", "c.txt"]);
}

#[gpui::test]
fn picking_in_one_list_does_not_carry_over_to_the_other(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    git(&fixture.alpha, &["add", "a.txt", "b.txt"]);
    let (workspace, cx) = open(&fixture, cx);
    click_row(cx, "file-staged-a.txt", Modifiers::none());
    click_row(cx, "file-unstaged-c.txt", shift());
    assert_eq!(
        picked(&workspace, cx),
        ["c.txt"],
        "Shift does not cross lists"
    );
}

#[gpui::test]
fn a_shift_click_without_an_anchor_picks_only_that_file(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);

    click_row(cx, "file-unstaged-c.txt", shift());

    assert_eq!(picked(&workspace, cx), ["c.txt"]);
}

#[gpui::test]
fn clicking_a_row_opens_its_diff_and_the_diff_follows_the_file_across_lists(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open_beta(&fixture, cx);

    click_row(cx, "file-unstaged-draft.txt", Modifiers::none());
    assert_eq!(viewing_staged(&workspace, cx), Some(false));

    drag(cx, "file-unstaged-draft.txt", "list-staged");
    assert_eq!(viewing_staged(&workspace, cx), Some(true));
    drag(cx, "file-staged-draft.txt", "list-unstaged");
    assert_eq!(viewing_staged(&workspace, cx), Some(false));
}
