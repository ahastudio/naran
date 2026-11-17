//! What the platform asks of the input, and how the text is painted.

use gpui::{EntityInputHandler, TestAppContext, point, px, size};

use super::*;
use crate::text_input::editing::{UTF8_TO_UTF16, UTF16_TO_UTF8};
use crate::text_input::element::{plain_run, text_runs};

#[gpui::test]
fn a_range_given_in_utf16_is_replaced_and_a_point_past_the_text_has_no_letter(
    cx: &mut TestAppContext,
) {
    let (input, cx) = open_indented(cx, "a한b", 500.);
    let area = bounds(&input, cx);
    input.update_in(cx, |input, window, cx| {
        input.replace_text_in_range(Some(1..2), "x", window, cx);
        assert_eq!(input.content, "axb");
        let inside = point(area.left() + px(1.), area.center().y);
        assert_eq!(input.character_index_for_point(inside, window, cx), None);
    });
}

#[gpui::test]
fn composed_text_is_marked_then_confirmed(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "ab");
    input.update_in(cx, |input, window, cx| {
        input.replace_and_mark_text_in_range(Some(1..1), "ㅎ", Some(0..1), window, cx);
        assert_eq!(input.content, "aㅎb");
        assert_eq!(input.marked_range, Some(1..4));
        assert_eq!(input.marked_text_range(window, cx), Some(1..2));
        assert_eq!(input.selected_range, 1..2);
        input.replace_and_mark_text_in_range(None, "하", None, window, cx);
        assert_eq!(input.content, "a하b");
        assert_eq!(input.selected_range, 4..4);
        input.unmark_text(window, cx);
        assert_eq!(input.marked_text_range(window, cx), None);
        input.replace_and_mark_text_in_range(None, "", None, window, cx);
        assert_eq!(input.marked_range, None);
    });
    cx.run_until_parked();
}

#[gpui::test]
fn the_text_of_a_range_and_its_place_are_answered_in_utf16(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "a한b");
    let area = bounds(&input, cx);
    input.update_in(cx, |input, window, cx| {
        let mut actual = None;
        assert_eq!(
            input
                .text_for_range(1..2, &mut actual, window, cx)
                .as_deref(),
            Some("한")
        );
        assert_eq!(actual, Some(1..2));
        let selected = input.selected_text_range(false, window, cx).unwrap();
        assert_eq!((selected.range, selected.reversed), (0..3, false));
        let range = input.bounds_for_range(1..2, area, window, cx).unwrap();
        assert!(range.left() > area.left() && range.right() > range.left());
        let inside = point(area.left() + px(1.), area.center().y);
        assert_eq!(input.character_index_for_point(inside, window, cx), Some(0));
        let far = point(px(-1000.), px(-1000.));
        assert_eq!(input.character_index_for_point(far, window, cx), None);
    });
}

#[gpui::test]
fn without_a_painted_line_nothing_can_be_located(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abc");
    input.update_in(cx, |input, window, cx| {
        input.last_layout = None;
        input.last_bounds = None;
        let area = Bounds::new(point(px(0.), px(0.)), size(px(10.), px(10.)));
        assert!(input.bounds_for_range(0..1, area, window, cx).is_none());
        assert!(
            input
                .character_index_for_point(point(px(1.), px(1.)), window, cx)
                .is_none()
        );
    });
}

#[gpui::test]
fn the_cursor_and_the_selection_are_painted_with_their_own_quads(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abc");
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    cx.simulate_keystrokes("shift-left");
    cx.run_until_parked();
    assert_eq!(selection(&input, cx), 2..3);
}

#[test]
fn offsets_convert_between_utf8_and_utf16() {
    let text = "a한😀b";
    assert_eq!(UTF8_TO_UTF16.convert(text, 0), 0);
    assert_eq!(UTF8_TO_UTF16.convert(text, 1), 1);
    assert_eq!(UTF8_TO_UTF16.convert(text, 4), 2);
    assert_eq!(UTF8_TO_UTF16.convert(text, 8), 4);
    assert_eq!(UTF16_TO_UTF8.convert(text, 4), 8);
    assert_eq!(UTF16_TO_UTF8.convert(text, 100), 9);
}

#[test]
fn the_runs_split_around_the_composed_part() {
    let text: SharedString = "abcdef".into();
    let font = gpui::font("Menlo");
    let color = gpui::hsla(0., 0., 0.5, 1.);

    let run = plain_run(text.len(), color, font);

    let plain = text_runs(run.clone(), None);
    let marked = text_runs(run.clone(), Some(&(2..4)));
    let at_start = text_runs(run, Some(&(0..2)));

    assert_eq!(plain.iter().map(|run| run.len).collect::<Vec<_>>(), [6]);
    assert_eq!(
        marked.iter().map(|run| run.len).collect::<Vec<_>>(),
        [2, 2, 2]
    );
    assert!(marked[1].underline.is_some() && marked[0].underline.is_none());
    assert_eq!(
        at_start.iter().map(|run| run.len).collect::<Vec<_>>(),
        [2, 4]
    );
}
