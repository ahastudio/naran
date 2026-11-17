//! Single-line text input.
//!
//! Adapted from the `input` example that ships with GPUI 0.2.2
//! (<https://github.com/zed-industries/zed>, crates/gpui/examples/input.rs),
//! licensed under the Apache License 2.0. Changes: Enter and Escape emit
//! events, colors come from the caller, and the example window is removed.
//! IME composition (for example Korean) works through the marked range.

mod editing;
mod element;
mod platform;

use std::ops::Range;

use gpui::{
    App, Bounds, Context, CursorStyle, Div, EventEmitter, FocusHandle, Focusable, Hsla, KeyBinding,
    MouseButton, Pixels, ShapedLine, SharedString, Window, actions, div, prelude::*,
};

use element::TextElement;

actions!(
    text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
        Submit,
        Cancel,
    ]
);

const CONTEXT: &str = "TextInput";

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some(CONTEXT)),
        KeyBinding::new("delete", Delete, Some(CONTEXT)),
        KeyBinding::new("left", Left, Some(CONTEXT)),
        KeyBinding::new("right", Right, Some(CONTEXT)),
        KeyBinding::new("shift-left", SelectLeft, Some(CONTEXT)),
        KeyBinding::new("shift-right", SelectRight, Some(CONTEXT)),
        KeyBinding::new("cmd-a", SelectAll, Some(CONTEXT)),
        KeyBinding::new("cmd-v", Paste, Some(CONTEXT)),
        KeyBinding::new("cmd-c", Copy, Some(CONTEXT)),
        KeyBinding::new("cmd-x", Cut, Some(CONTEXT)),
        KeyBinding::new("home", Home, Some(CONTEXT)),
        KeyBinding::new("end", End, Some(CONTEXT)),
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, Some(CONTEXT)),
        KeyBinding::new("enter", Submit, Some(CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(CONTEXT)),
    ]);
}

pub enum InputEvent {
    Submit(String),
    Cancel,
}

/// How an input looks while it is empty, and the color of its cursor and
/// selection.
pub struct InputLook {
    pub placeholder: SharedString,
    pub placeholder_color: Hsla,
    pub cursor_color: Hsla,
}

pub struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    look: InputLook,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
}

impl EventEmitter<InputEvent> for TextInput {}

impl TextInput {
    pub fn new(content: &str, look: InputLook, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: content.to_string().into(),
            look,
            selected_range: 0..content.len(),
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        }
    }

    /// Keeps what was painted last, for turning a point into an offset.
    fn remember_layout(&mut self, line: ShapedLine, bounds: Bounds<Pixels>) {
        self.last_layout = Some(line);
        self.last_bounds = Some(bounds);
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let frame = div()
            .flex()
            .flex_1()
            .key_context(CONTEXT)
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam);
        let frame = Self::on_mouse(Self::on_clipboard(Self::on_editing(frame, cx), cx), cx);
        frame.child(TextElement::new(cx.entity()))
    }
}

impl TextInput {
    fn on_editing(frame: Div, cx: &mut Context<Self>) -> Div {
        frame
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
    }

    fn on_clipboard(frame: Div, cx: &mut Context<Self>) -> Div {
        frame
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::cancel))
    }

    fn on_mouse(frame: Div, cx: &mut Context<Self>) -> Div {
        frame
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests;
