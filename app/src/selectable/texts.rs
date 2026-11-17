//! The two selectable views of the detail column, and the mouse handling
//! that turns presses and drags into a selection.

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, FocusHandle, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, Window, prelude::*,
};

use super::Region;

/// The two views whose text can be selected: the commit information and the
/// diff. Only one of them has a selection at a time.
#[derive(Clone)]
pub struct Texts {
    pub info: Region,
    pub diff: Region,
    /// The region pressed last, which Select All applies to.
    last: Rc<Cell<Which>>,
    /// Pressing text moves the keyboard focus here, so Copy reaches the window.
    focus: FocusHandle,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Which {
    #[default]
    Info,
    Diff,
}

impl Which {
    fn other(self) -> Self {
        match self {
            Self::Info => Self::Diff,
            Self::Diff => Self::Info,
        }
    }
}

impl Texts {
    pub fn new(focus: FocusHandle) -> Self {
        Self {
            info: Region::default(),
            diff: Region::default(),
            last: Rc::default(),
            focus,
        }
    }

    /// A press on the text of `which`: the other view loses its selection and
    /// the keyboard focus comes here.
    fn press(&self, which: Which, position: Point<Pixels>, window: &mut Window) {
        self.region(which.other()).clear();
        self.last.set(which);
        window.focus(&self.focus);
        self.region(which).press(position);
        window.refresh();
    }

    fn region(&self, which: Which) -> &Region {
        match which {
            Which::Info => &self.info,
            Which::Diff => &self.diff,
        }
    }

    /// Call at the start of every render, before the views are built.
    pub fn begin_frame(&self) {
        self.info.begin_frame();
        self.diff.begin_frame();
    }

    /// The selected text, for the clipboard.
    pub fn copied(&self) -> Option<String> {
        self.diff
            .selected_text()
            .or_else(|| self.info.selected_text())
    }

    pub fn select_all(&self) {
        self.region(self.last.get()).select_all();
    }
}

/// Makes the text lines inside `content` selectable with the mouse.
pub fn area(content: Div, texts: &Texts, which: Which) -> Div {
    let region = texts.region(which).clone();
    let texts = texts.clone();
    content
        .cursor(CursorStyle::IBeam)
        .on_mouse_down(
            MouseButton::Left,
            move |event: &MouseDownEvent, window, _| texts.press(which, event.position, window),
        )
        .on_mouse_move(drag_handler(&region))
        .on_mouse_up(MouseButton::Left, release_handler(&region))
        .on_mouse_up_out(MouseButton::Left, release_handler(&region))
}

fn drag_handler(region: &Region) -> impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static {
    let region = region.clone();
    move |event, window, _| {
        if region.drag(event.position) {
            window.refresh();
        }
    }
}

fn release_handler(region: &Region) -> impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static {
    let region = region.clone();
    move |_, window, _| {
        region.release();
        window.refresh();
    }
}

#[cfg(test)]
mod tests;
