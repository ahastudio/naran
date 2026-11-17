//! Scrolling: lists following the arrow keys and diffs moving sideways.

use super::*;

#[gpui::test]
fn the_arrow_keys_scroll_the_commit_list_to_follow_the_picked_row(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    add_commits(&fixture.alpha, 100);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();
    workspace.update(cx, |workspace, _| workspace.layout.pane = Pane::Commits);
    for _ in 0..40 {
        workspace.update(cx, |workspace, cx| workspace.step(1, cx));
        cx.run_until_parked();
    }
    let down = workspace.read_with(cx, |workspace, _| scrolled(&workspace.scrolls.history));
    assert!(
        down >= 600.,
        "the list followed the picked row down: scrolled {down}px"
    );

    for _ in 0..40 {
        workspace.update(cx, |workspace, cx| workspace.step(-1, cx));
        cx.run_until_parked();
    }
    let up = workspace.read_with(cx, |workspace, _| scrolled(&workspace.scrolls.history));
    assert_eq!(up, 0., "and back up to the first row");
}

#[gpui::test]
fn the_arrow_keys_scroll_the_file_lists_too(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    for index in 0..80 {
        std::fs::write(fixture.alpha.join(format!("f{index:02}.txt")), "x\n").unwrap();
    }
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();
    workspace.update(cx, |workspace, _| workspace.layout.pane = Pane::Files);
    for _ in 0..60 {
        workspace.update(cx, |workspace, cx| workspace.step(1, cx));
        cx.run_until_parked();
    }
    let down = workspace.read_with(cx, |workspace, _| scrolled(&workspace.scrolls.unstaged));
    assert!(
        down >= 600.,
        "the Unstaged files list followed the picked file: scrolled {down}px"
    );
}

#[gpui::test]
fn the_arrow_keys_scroll_the_files_of_a_past_commit(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    for index in 0..60 {
        std::fs::write(fixture.alpha.join(format!("g{index:02}.txt")), "y\n").unwrap();
    }
    git(&fixture.alpha, &["add", "-A"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "many files"]);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked(); // a clean repository starts on its top commit
    workspace.update(cx, |workspace, _| workspace.layout.pane = Pane::Files);
    for _ in 0..40 {
        workspace.update(cx, |workspace, cx| workspace.step(1, cx));
        cx.run_until_parked();
    }
    let down = workspace.read_with(cx, |workspace, _| scrolled(&workspace.scrolls.commit_files));
    assert!(
        down >= 400.,
        "the file list of the commit followed the picked file: scrolled {down}px"
    );
}

/// Turns the scroll wheel sideways over the middle of a view, the way a
/// trackpad swipe does, and gives how far the view moved to the right.
fn swipe_sideways(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
    handle: impl Fn(&Workspace) -> ScrollHandle,
) -> f32 {
    use gpui::{ScrollDelta, ScrollWheelEvent, TouchPhase, point, px};
    let bounds = workspace.read_with(cx, |workspace, _| handle(workspace).bounds());
    cx.simulate_event(ScrollWheelEvent {
        position: bounds.center(),
        delta: ScrollDelta::Pixels(point(px(-400.), px(0.))),
        modifiers: gpui::Modifiers::default(),
        touch_phase: TouchPhase::Moved,
    });
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| -f32::from(handle(workspace).offset().x))
}

/// A line much wider than the window, and a short one.
fn a_long_line() -> String {
    format!("{}\n", "wide ".repeat(200))
}

#[gpui::test]
fn a_diff_with_a_long_line_scrolls_sideways(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("wide.txt"), a_long_line()).unwrap();
    std::fs::write(fixture.alpha.join("short.txt"), "short\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "wide.txt".into(),
            Staging::Unstaged,
            state('?', '?', false),
            cx,
        );
    });
    cx.run_until_parked();
    let wide = swipe_sideways(&workspace, cx, |workspace| {
        workspace.scrolls.diff_sideways.clone()
    });
    assert_eq!(
        wide, 400.,
        "the diff moved with the swipe to show the rest of the long line"
    );

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "short.txt".into(),
            Staging::Unstaged,
            state('?', '?', false),
            cx,
        );
    });
    cx.run_until_parked();
    workspace.update(cx, |workspace, _| {
        workspace
            .scrolls
            .diff_sideways
            .set_offset(gpui::point(px(0.), px(0.)));
    });
    let short = swipe_sideways(&workspace, cx, |workspace| {
        workspace.scrolls.diff_sideways.clone()
    });
    assert_eq!(short, 0., "nothing to scroll when every line fits");
}

#[gpui::test]
fn the_diff_of_a_past_commit_scrolls_sideways_too(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("wide.txt"), a_long_line()).unwrap();
    git(&fixture.alpha, &["add", "wide.txt"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "wide"]);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked(); // a clean repository starts on its top commit, whose file opens by itself

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(
            workspace
                .shown
                .inspecting
                .as_ref()
                .and_then(|inspecting| inspecting.file.as_deref()),
            Some("wide.txt")
        );
    });
    let wide = swipe_sideways(&workspace, cx, |workspace| {
        workspace.scrolls.diff_sideways.clone()
    });
    assert_eq!(
        wide, 400.,
        "the diff moved with the swipe to show the rest of the long line"
    );
}

#[gpui::test]
fn stacked_diffs_scroll_sideways_too(cx: &mut TestAppContext) {
    use gpui::Modifiers;
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("a.txt"), "a\n").unwrap();
    std::fs::write(fixture.alpha.join("b.txt"), a_long_line()).unwrap();
    let (workspace, cx) = open(&fixture, cx);
    let untracked = state('?', '?', false);
    workspace.update(cx, |workspace, cx| {
        workspace.click_file(
            ListedFile::new("a.txt", Staging::Unstaged, untracked),
            Modifiers::default(),
            cx,
        );
    });
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    workspace.update(cx, |workspace, cx| {
        workspace.click_file(
            ListedFile::new("b.txt", Staging::Unstaged, untracked),
            shift,
            cx,
        );
    });
    cx.run_until_parked();
    assert!(stack(&workspace, cx).is_some());

    let wide = swipe_sideways(&workspace, cx, |workspace| {
        workspace.scrolls.stacked.0.borrow().base_handle.clone()
    });
    assert_eq!(
        wide, 400.,
        "the stacked diffs moved with the swipe to show the rest of the long line"
    );
}

#[gpui::test]
fn another_file_opens_at_the_left_edge_of_its_diff(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("wide.txt"), a_long_line()).unwrap();
    std::fs::write(fixture.alpha.join("wider.txt"), a_long_line()).unwrap();
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "wide.txt".into(),
            Staging::Unstaged,
            state('?', '?', false),
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(
        swipe_sideways(&workspace, cx, |workspace| workspace
            .scrolls
            .diff_sideways
            .clone()),
        400.
    );

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "wider.txt".into(),
            Staging::Unstaged,
            state('?', '?', false),
            cx,
        );
    });
    cx.run_until_parked();
    let offset = workspace.read_with(cx, |workspace, _| {
        f32::from(workspace.scrolls.diff_sideways.offset().x)
    });
    assert_eq!(offset, 0.);
}

#[gpui::test]
fn another_file_of_a_past_commit_opens_at_the_left_edge_too(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("wide.txt"), a_long_line()).unwrap();
    std::fs::write(fixture.alpha.join("wider.txt"), a_long_line()).unwrap();
    git(&fixture.alpha, &["add", "-A"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "wide files"]);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();
    assert_eq!(
        swipe_sideways(&workspace, cx, |workspace| workspace
            .scrolls
            .diff_sideways
            .clone()),
        400.
    );

    workspace.update(cx, |workspace, cx| {
        workspace.open_commit_file("wider.txt".into(), cx);
    });
    cx.run_until_parked();
    let offset = workspace.read_with(cx, |workspace, _| {
        f32::from(workspace.scrolls.diff_sideways.offset().x)
    });
    assert_eq!(offset, 0.);
}
