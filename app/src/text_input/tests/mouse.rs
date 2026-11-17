//! The mouse places the cursor and drags a selection.

use gpui::{Modifiers, Point, TestAppContext, VisualTestContext, point, px};

use super::*;

#[gpui::test]
fn the_mouse_places_the_cursor_and_drags_a_selection(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abcdef");
    let area = bounds(&input, cx);
    let at = |x: f32| point(area.left() + px(x), area.center().y);

    cx.simulate_mouse_down(at(0.), MouseButton::Left, Modifiers::none());
    assert_eq!(selection(&input, cx), 0..0);
    cx.simulate_mouse_move(at(1000.), MouseButton::Left, Modifiers::none());
    assert_eq!(selection(&input, cx), 0..6);
    cx.simulate_mouse_up(at(1000.), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(at(0.), MouseButton::Left, Modifiers::none());
    assert_eq!(
        selection(&input, cx),
        0..6,
        "moving after release selects nothing"
    );
}

#[gpui::test]
fn shift_with_a_click_extends_the_selection(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abcdef");
    let area = bounds(&input, cx);
    cx.simulate_keystrokes("home");

    cx.simulate_mouse_down(
        point(area.left() + px(1000.), area.center().y),
        MouseButton::Left,
        Modifiers::shift(),
    );

    assert_eq!(selection(&input, cx), 0..6);
}

#[gpui::test]
fn a_point_above_or_below_the_line_means_an_end_of_the_text(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abcdef");
    let area = bounds(&input, cx);
    let index = |position: Point<Pixels>, cx: &mut VisualTestContext| {
        input.read_with(cx, |input, _| input.index_for_mouse_position(position))
    };

    assert_eq!(
        index(point(area.left() + px(30.), area.top() - px(5.)), cx),
        0
    );
    assert_eq!(index(point(area.left(), area.bottom() + px(5.)), cx), 6);
    assert_eq!(
        index(point(area.left() + px(1000.), area.center().y), cx),
        6
    );
}

#[gpui::test]
fn a_click_in_an_empty_input_stays_at_the_start(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "");
    let area = bounds(&input, cx);

    cx.simulate_mouse_down(area.center(), MouseButton::Left, Modifiers::none());

    assert_eq!(selection(&input, cx), 0..0);
}
