use std::cell::RefCell;
use std::rc::Rc;

use gpui::{ClipboardItem, Entity, EntityInputHandler, TestAppContext, VisualTestContext, px};

use super::*;

mod mouse;
mod platform;

/// A window as wide as the screen, so the input has room to be pointed at.
struct Host(Entity<TextInput>, f32);

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .pl(px(self.1))
            .child(self.0.clone())
    }
}

fn open<'a>(
    cx: &'a mut TestAppContext,
    content: &str,
) -> (Entity<TextInput>, &'a mut VisualTestContext) {
    open_indented(cx, content, 0.)
}

/// The input starts `indent` pixels from the left edge of the window.
fn open_indented<'a>(
    cx: &'a mut TestAppContext,
    content: &str,
    indent: f32,
) -> (Entity<TextInput>, &'a mut VisualTestContext) {
    cx.update(bind_keys);
    let content = content.to_string();
    let (host, cx) = cx.add_window_view(|window, cx| {
        let color = gpui::hsla(0., 0., 0.5, 1.);
        let input = cx.new(|cx| {
            TextInput::new(
                &content,
                InputLook {
                    placeholder: "Name".into(),
                    placeholder_color: color,
                    cursor_color: color,
                },
                cx,
            )
        });
        window.focus(&input.read(cx).focus_handle);
        Host(input, indent)
    });
    cx.run_until_parked();
    (host.read_with(cx, |host, _| host.0.clone()), cx)
}

fn content(input: &Entity<TextInput>, cx: &mut VisualTestContext) -> String {
    input.read_with(cx, |input, _| input.content.to_string())
}

fn selection(input: &Entity<TextInput>, cx: &mut VisualTestContext) -> Range<usize> {
    input.read_with(cx, |input, _| input.selected_range.clone())
}

fn bounds(input: &Entity<TextInput>, cx: &mut VisualTestContext) -> Bounds<Pixels> {
    input.read_with(cx, |input, _| input.last_bounds.unwrap())
}

fn events(input: &Entity<TextInput>, cx: &mut VisualTestContext) -> Rc<RefCell<Vec<String>>> {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let log = seen.clone();
    cx.update(|_, cx| {
        cx.subscribe(input, move |_, event: &InputEvent, _| {
            log.borrow_mut().push(match event {
                InputEvent::Submit(text) => format!("submit {text}"),
                InputEvent::Cancel => "cancel".to_string(),
            });
        })
        .detach();
    });
    seen
}

#[gpui::test]
fn the_text_starts_selected_and_typing_replaces_it(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abc");
    assert_eq!(selection(&input, cx), 0..3);

    cx.simulate_input("x");

    assert_eq!(content(&input, cx), "x");
    assert_eq!(selection(&input, cx), 1..1);
}

#[gpui::test]
fn the_cursor_moves_by_letter_and_to_both_ends(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "a한b");
    for (keys, expected) in [
        ("end", 5..5),
        ("left", 4..4),
        ("left", 1..1),
        ("right", 4..4),
        ("home", 0..0),
        ("left", 0..0),
        ("right", 1..1),
    ] {
        cx.simulate_keystrokes(keys);
        assert_eq!(selection(&input, cx), expected, "{keys}");
    }
}

#[gpui::test]
fn a_selection_collapses_to_the_side_the_cursor_moves_to(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abc");
    cx.simulate_keystrokes("left");
    assert_eq!(selection(&input, cx), 0..0);
    cx.simulate_keystrokes("cmd-a right");
    assert_eq!(selection(&input, cx), 3..3);
}

#[gpui::test]
fn the_selection_grows_both_ways_and_turns_around(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abcd");
    cx.simulate_keystrokes("end shift-left shift-left");
    assert_eq!(selection(&input, cx), 2..4);
    cx.simulate_keystrokes("shift-right shift-right shift-right");
    assert_eq!(selection(&input, cx), 4..4);
    cx.simulate_keystrokes("shift-left");
    assert_eq!(selection(&input, cx), 3..4);
    cx.simulate_keystrokes("cmd-a");
    assert_eq!(selection(&input, cx), 0..4);
}

#[gpui::test]
fn backspace_and_delete_remove_a_letter_or_the_selection(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abcd");
    cx.simulate_keystrokes("end backspace");
    assert_eq!(content(&input, cx), "abc");
    cx.simulate_keystrokes("home delete");
    assert_eq!(content(&input, cx), "bc");
    cx.simulate_keystrokes("cmd-a delete");
    assert_eq!(content(&input, cx), "");
    cx.simulate_input("bc");
    cx.simulate_keystrokes("cmd-a backspace");
    assert_eq!(content(&input, cx), "");
    cx.simulate_keystrokes("backspace delete");
    assert_eq!(content(&input, cx), "");
}

#[gpui::test]
fn copy_cut_and_paste_use_the_clipboard(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abc");
    let clipboard =
        |cx: &mut VisualTestContext| cx.read_from_clipboard().and_then(|item| item.text());

    cx.simulate_keystrokes("cmd-c");
    assert_eq!(clipboard(cx).as_deref(), Some("abc"));
    cx.simulate_keystrokes("home cmd-c");
    assert_eq!(
        clipboard(cx).as_deref(),
        Some("abc"),
        "nothing selected: nothing copied"
    );
    cx.simulate_keystrokes("cmd-a cmd-x");
    assert_eq!(content(&input, cx), "");
    cx.simulate_keystrokes("cmd-x");
    cx.write_to_clipboard(ClipboardItem::new_string("one\ntwo".into()));
    cx.simulate_keystrokes("cmd-v");
    assert_eq!(content(&input, cx), "one two");
    cx.write_to_clipboard(ClipboardItem::new_string("ignored".into()));
    cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("x".into())));
}

#[gpui::test]
fn pasting_with_an_empty_clipboard_does_nothing(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "abc");
    cx.simulate_keystrokes("home cmd-v");
    assert_eq!(content(&input, cx), "abc");
}

#[gpui::test]
fn enter_submits_and_escape_cancels(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "name");
    let seen = events(&input, cx);

    cx.simulate_keystrokes("enter escape");

    assert_eq!(*seen.borrow(), ["submit name", "cancel"]);
}

#[gpui::test]
fn enter_while_composing_does_not_submit(cx: &mut TestAppContext) {
    let (input, cx) = open(cx, "");
    let seen = events(&input, cx);
    input.update_in(cx, |input, window, cx| {
        input.replace_and_mark_text_in_range(None, "ㅎ", None, window, cx);
    });

    cx.simulate_keystrokes("enter");

    assert!(seen.borrow().is_empty());
}

/// The test window has no character palette; reaching it is what the key does.
#[gpui::test]
#[should_panic(expected = "not implemented")]
fn the_character_palette_key_asks_the_window_for_the_palette(cx: &mut TestAppContext) {
    let (_input, cx) = open(cx, "abc");
    cx.simulate_keystrokes("ctrl-cmd-space");
}
