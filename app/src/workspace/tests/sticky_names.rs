//! File names that stay above their diffs while they scroll.

use super::*;

/// Turns the scroll wheel down over the middle of a view.
fn swipe_down(cx: &mut VisualTestContext, bounds: gpui::Bounds<Pixels>, by: f32) {
    use gpui::{ScrollDelta, ScrollWheelEvent, TouchPhase, point, px};
    cx.simulate_event(ScrollWheelEvent {
        position: bounds.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-by))),
        modifiers: gpui::Modifiers::default(),
        touch_phase: TouchPhase::Moved,
    });
    cx.run_until_parked();
}

fn many_lines(count: usize) -> String {
    many_lines_from(0, count)
}

#[gpui::test]
fn the_file_name_stays_above_the_diff_while_it_scrolls(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("tall.txt"), many_lines(200)).unwrap();
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "tall.txt".into(),
            Staging::Unstaged,
            state('?', '?', false),
            cx,
        );
    });
    cx.run_until_parked();

    let before = cx.debug_bounds("diff-name").expect("the name is shown");
    let lines = workspace.read_with(cx, |workspace, _| workspace.scrolls.diff_sideways.bounds());
    let area = cx.debug_bounds("diff-lines").unwrap();
    swipe_down(cx, area, 600.);
    let after = cx
        .debug_bounds("diff-name")
        .expect("the name is still shown");
    assert_eq!(after.origin, before.origin, "the name did not move");
    let moved = workspace.read_with(cx, |workspace, _| {
        workspace.scrolls.diff_sideways.bounds().origin.y
    });
    assert!(moved < lines.origin.y, "the lines moved up under it");
}

#[gpui::test]
fn the_name_of_a_past_commit_file_stays_above_its_diff_too(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("tall.txt"), many_lines(200)).unwrap();
    git(&fixture.alpha, &["add", "tall.txt"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "tall"]);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();

    let before = cx.debug_bounds("diff-name").expect("the name is shown");
    let lines = workspace.read_with(cx, |workspace, _| workspace.scrolls.diff_sideways.bounds());
    let area = cx.debug_bounds("diff-lines").unwrap();
    swipe_down(cx, area, 600.);
    assert_eq!(
        cx.debug_bounds("diff-name").unwrap().origin,
        before.origin,
        "the name did not move"
    );
    let moved = workspace.read_with(cx, |workspace, _| {
        workspace.scrolls.diff_sideways.bounds().origin.y
    });
    assert!(moved < lines.origin.y, "the lines moved up under it");
}

#[gpui::test]
fn stacked_diffs_keep_the_name_of_the_file_at_the_top_until_the_next_file(cx: &mut TestAppContext) {
    use gpui::Modifiers;
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("a.txt"), many_lines(40)).unwrap();
    std::fs::write(fixture.alpha.join("b.txt"), many_lines(40)).unwrap();
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
    let rows = stack(&workspace, cx).unwrap();
    let second = rows
        .iter()
        .rposition(|row| matches!(row, detail::StackRow::File { .. }))
        .unwrap();
    let list = workspace.read_with(cx, |workspace, _| {
        workspace.scrolls.stacked.0.borrow().base_handle.bounds()
    });

    assert!(
        cx.debug_bounds("sticky-head-0").is_none(),
        "at the top the head is in its own place"
    );
    swipe_down(cx, list, 200.);
    let stuck = cx
        .debug_bounds("sticky-head-0")
        .expect("a.txt stays at the top");
    assert_eq!(stuck.origin.y, list.origin.y);

    swipe_down(
        cx,
        list,
        f32::from(u16::try_from(second).unwrap()) * 20. - 200.,
    );
    // GPUI keeps the bounds of earlier frames in two buffers and never clears
    // them, so "no bounds" depends on how many frames were drawn since. Only
    // the head that is drawn now is asserted; one sticky head is drawn at a
    // time (`sticky_head` is tested on its own).
    let head = format!("sticky-head-{second}");
    let came_in = cx
        .debug_bounds(Box::leak(head.into_boxed_str()))
        .expect("b.txt is the head at the top");
    assert_eq!(came_in.origin.y, list.origin.y, "b.txt has fully come in");
}
