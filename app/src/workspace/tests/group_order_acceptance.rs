//! Acceptance scenario of reordering the groups by dragging a row, driven
//! through mouse events.

use super::groups_and_repositories_acceptance::group_names;
use super::*;

#[gpui::test]
fn dragging_a_group_row_with_the_mouse_reorders_the_groups(cx: &mut TestAppContext) {
    use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, px};
    let fixture = fixture(&[("a", &[0]), ("b", &[1]), ("c", &[2])]);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(group_names(&workspace, cx), ["a", "b", "c"]);

    let first = cx.debug_bounds("group-0").expect("the first group row");
    let third = cx.debug_bounds("group-2").expect("the third group row");
    let (from, to) = (third.center(), first.center());
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position: from,
        modifiers: Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    for step in [10., 30., 60.] {
        cx.simulate_event(MouseMoveEvent {
            position: gpui::point(from.x, from.y - px(step)),
            pressed_button: Some(MouseButton::Left),
            modifiers: Modifiers::default(),
        });
        cx.run_until_parked();
    }
    cx.simulate_event(MouseMoveEvent {
        position: to,
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::default(),
    });
    cx.run_until_parked();
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: to,
        modifiers: Modifiers::default(),
        click_count: 1,
    });
    cx.run_until_parked();
    assert_eq!(group_names(&workspace, cx), ["c", "a", "b"]);
}
