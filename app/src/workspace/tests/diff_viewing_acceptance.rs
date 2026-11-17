//! Acceptance tests of viewing changes that drive the screen with wheel
//! swipes and mouse drags.

use gpui::{Bounds, Modifiers, MouseButton, ScrollDelta, ScrollWheelEvent, TouchPhase};

use super::*;

fn swipe(cx: &mut VisualTestContext, bounds: Bounds<Pixels>, by: gpui::Point<Pixels>) {
    cx.simulate_event(ScrollWheelEvent {
        position: bounds.center(),
        delta: ScrollDelta::Pixels(by),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
    });
    cx.run_until_parked();
}

fn two_wide_files(fixture: &Fixture) {
    let rows = "row\n".repeat(40);
    let wide = format!("{}\n{rows}", "wide ".repeat(200));
    std::fs::write(fixture.alpha.join("a.txt"), &wide).unwrap();
    std::fs::write(fixture.alpha.join("b.txt"), &wide).unwrap();
}

fn pick_both(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) {
    let untracked = state('?', '?', false);
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    for (name, modifiers) in [("a.txt", Modifiers::default()), ("b.txt", shift)] {
        workspace.update(cx, |workspace, cx| {
            let file = ListedFile::new(name, Staging::Unstaged, untracked);
            workspace.click_file(file, modifiers, cx);
        });
    }
    cx.run_until_parked();
}

#[gpui::test]
fn the_sticky_file_head_stays_at_the_left_edge_while_the_stack_scrolls_sideways(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    two_wide_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);
    pick_both(&workspace, cx);
    let handle = workspace.read_with(cx, |workspace, _| {
        workspace.scrolls.stacked.0.borrow().base_handle.clone()
    });
    let list = handle.bounds();

    swipe(cx, list, gpui::point(px(0.), px(-200.)));
    let before = cx.debug_bounds("sticky-head-0").expect("a.txt sticks");
    swipe(cx, list, gpui::point(px(-400.), px(0.)));
    assert_eq!(
        f32::from(handle.offset().x),
        -400.,
        "the list moved sideways"
    );
    let after = cx
        .debug_bounds("sticky-head-0")
        .expect("a.txt still sticks");
    assert_eq!(after.origin.x, before.origin.x, "the head did not move");
    assert_eq!(after.origin.x, list.origin.x, "it is at the left edge");
}

#[gpui::test]
fn dragging_over_the_text_of_a_diff_selects_it_for_copying(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("a.txt"), "first\nsecond\nthird\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        let untracked = state('?', '?', false);
        workspace.open_file("a.txt".into(), Staging::Unstaged, untracked, cx);
    });
    cx.run_until_parked();
    let rows: Vec<Bounds<Pixels>> = (0..8)
        .filter_map(|index| {
            let id: &'static str = Box::leak(format!("diff-line-{index}").into_boxed_str());
            cx.debug_bounds(id)
        })
        .collect();
    assert!(rows.len() >= 3, "the lines are shown: {}", rows.len());
    let from = rows[0].origin + gpui::point(px(90.), px(4.));
    let to = rows[2].origin + gpui::point(px(300.), px(4.));
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    cx.run_until_parked();
    let copied = workspace.read_with(cx, |workspace, _| workspace.texts.copied());
    let copied = copied.expect("something is selected");
    assert!(copied.contains("second"), "{copied:?}");
}

#[gpui::test]
fn an_image_among_the_picked_files_shows_a_note_in_the_stack(cx: &mut TestAppContext) {
    use super::detail::StackRow;
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("a.txt"), "a\n").unwrap();
    std::fs::write(fixture.alpha.join("pic.png"), TINY_PNG).unwrap();
    let (workspace, cx) = open(&fixture, cx);
    let untracked = state('?', '?', false);
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    for (name, modifiers) in [("a.txt", Modifiers::default()), ("pic.png", shift)] {
        workspace.update(cx, |workspace, cx| {
            let file = ListedFile::new(name, Staging::Unstaged, untracked);
            workspace.click_file(file, modifiers, cx);
        });
    }
    cx.run_until_parked();
    let rows = stack(&workspace, cx).expect("two files are stacked");
    let notes = rows
        .iter()
        .filter(|row| matches!(row, StackRow::Note(note) if note.starts_with("An image")))
        .count();
    assert_eq!(notes, 1, "the image has a note instead of lines");
}

#[gpui::test]
fn shift_with_the_arrow_keys_grows_the_picked_files_of_a_commit(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    for file in ["a.txt", "b.txt", "c.txt"] {
        std::fs::write(fixture.alpha.join(file), format!("{file}\n")).unwrap();
    }
    git(&fixture.alpha, &["add", "."]);
    git(&fixture.alpha, &["commit", "-q", "-m", "three files"]);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();
    workspace.update(cx, |workspace, cx| {
        workspace.click_commit_file("a.txt".into(), Modifiers::default(), cx);
        workspace.focus_pane(Pane::Files);
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("shift-down shift-down");
    cx.run_until_parked();
    let picked = workspace.read_with(cx, |workspace, _| {
        workspace.shown.inspecting.as_ref().unwrap().picked.len()
    });
    assert_eq!(picked, 3, "two Shift+Down presses grow the picks to three");
    cx.simulate_keystrokes("shift-up");
    cx.run_until_parked();
    let picked = workspace.read_with(cx, |workspace, _| {
        workspace.shown.inspecting.as_ref().unwrap().picked.len()
    });
    assert_eq!(picked, 2, "Shift+Up shrinks them back toward the anchor");
}

type Point = (usize, f32);

fn drag_across_lines(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
    (from, to): (Point, Point),
) -> Option<String> {
    let rows: Vec<Bounds<Pixels>> = (0..8)
        .filter_map(|index| {
            let id: &'static str = Box::leak(format!("diff-line-{index}").into_boxed_str());
            cx.debug_bounds(id)
        })
        .collect();
    let at = |(row, x): Point| rows[row].origin + gpui::point(px(x), px(4.));
    cx.simulate_mouse_down(at(from), MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(at(to), MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(at(to), MouseButton::Left, Modifiers::default());
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| workspace.texts.copied())
}

fn open_three_lines<'a>(
    fixture: &Fixture,
    cx: &'a mut TestAppContext,
) -> (Entity<Workspace>, &'a mut VisualTestContext) {
    std::fs::write(fixture.alpha.join("a.txt"), "first\nsecond\nthird\n").unwrap();
    let (workspace, cx) = open(fixture, cx);
    workspace.update(cx, |workspace, cx| {
        let untracked = state('?', '?', false);
        workspace.open_file("a.txt".into(), Staging::Unstaged, untracked, cx);
    });
    cx.run_until_parked();
    (workspace, cx)
}

#[gpui::test]
fn a_drag_over_three_lines_copies_the_cut_of_each_line(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open_three_lines(&fixture, cx);
    let copied = drag_across_lines(&workspace, cx, ((0, 90.), (2, 300.)));
    assert_eq!(copied.as_deref(), Some("rst\nsecond\nthird"));
}

#[gpui::test]
fn a_drag_upwards_copies_the_same_text_as_a_drag_downwards(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open_three_lines(&fixture, cx);
    let down = drag_across_lines(&workspace, cx, ((0, 90.), (2, 300.)));
    let up = drag_across_lines(&workspace, cx, ((2, 300.), (0, 90.)));
    assert!(down.is_some());
    assert_eq!(up, down);
}

#[gpui::test]
fn a_plain_click_on_a_diff_copies_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open_three_lines(&fixture, cx);
    let row = cx.debug_bounds("diff-line-0").expect("the first line");
    let click = row.origin + gpui::point(px(90.), px(4.));
    cx.simulate_click(click, Modifiers::default());
    cx.run_until_parked();
    let copied = workspace.read_with(cx, |workspace, _| workspace.texts.copied());
    assert_eq!(copied, None);
}
