//! Moving the cursor, selecting, and editing with the keys, the mouse and
//! the clipboard.

use std::ops::Range;

use gpui::{
    ClipboardItem, Context, EntityInputHandler, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, Window,
};
use unicode_segmentation::UnicodeSegmentation;

use super::{
    Backspace, Cancel, Copy, Cut, Delete, End, Home, InputEvent, Left, Paste, Right, SelectAll,
    SelectLeft, SelectRight, ShowCharacterPalette, Submit, TextInput,
};

impl TextInput {
    pub(super) fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    pub(super) fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    pub(super) fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    pub(super) fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    pub(super) fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    pub(super) fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    pub(super) fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    pub(super) fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(super) fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(super) fn submit(&mut self, _: &Submit, _: &mut Window, cx: &mut Context<Self>) {
        // Enter also confirms an IME composition; only submit when nothing is being composed.
        if self.marked_range.is_none() {
            cx.emit(InputEvent::Submit(self.content.to_string()));
        }
    }

    #[expect(clippy::unused_self, reason = "an action handler takes the input")]
    pub(super) fn cancel(&mut self, _: &Cancel, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(InputEvent::Cancel);
    }

    pub(super) fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    pub(super) fn on_mouse_up(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        _: &mut Context<Self>,
    ) {
        self.is_selecting = false;
    }

    pub(super) fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    #[expect(clippy::unused_self, reason = "an action handler takes the input")]
    pub(super) fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    pub(super) fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text.replace('\n', " "), window, cx);
        }
    }

    pub(super) fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    pub(super) fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify();
    }

    pub(super) fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub(super) fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        let Some((bounds, line)) = self
            .last_bounds
            .as_ref()
            .zip(self.last_layout.as_ref())
            .filter(|_| !self.content.is_empty())
        else {
            return 0;
        };
        match (position.y < bounds.top(), position.y > bounds.bottom()) {
            (true, _) => 0,
            (_, true) => self.content.len(),
            _ => line.closest_index_for_x(position.x - bounds.left()),
        }
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.move_edge(offset);
        self.keep_range_ordered();
        cx.notify();
    }

    /// Moves the end of the selection that follows the pointer or the cursor.
    fn move_edge(&mut self, offset: usize) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
    }

    /// When the moving end passed the other one, the range turns around.
    fn keep_range_ordered(&mut self) {
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        UTF16_TO_UTF8.convert(&self.content, offset)
    }

    pub(super) fn offset_to_utf16(&self, offset: usize) -> usize {
        UTF8_TO_UTF16.convert(&self.content, offset)
    }

    pub(super) fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub(super) fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(index, _)| (index < offset).then_some(index))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(index, _)| (index > offset).then_some(index))
            .unwrap_or(self.content.len())
    }
}

/// Adds the size of one character to `count`; tells the count before it.
fn advance(count: &mut usize, character: char, counted: fn(char) -> usize) -> usize {
    let before = *count;
    *count += counted(character);
    before
}

/// Turns an offset counted in one encoding into the same place counted in
/// another: the platform speaks UTF-16, the text is UTF-8.
pub(super) struct Conversion {
    /// The size of a character in the encoding the offset is counted in.
    counted: fn(char) -> usize,
    /// The size of a character in the encoding the result is counted in.
    produced: fn(char) -> usize,
}

pub(super) const UTF8_TO_UTF16: Conversion = Conversion {
    counted: char::len_utf8,
    produced: char::len_utf16,
};

pub(super) const UTF16_TO_UTF8: Conversion = Conversion {
    counted: char::len_utf16,
    produced: char::len_utf8,
};

impl Conversion {
    /// Counts the characters whose size adds up to `offset`, and adds up
    /// their size in the other encoding.
    pub(super) fn convert(&self, content: &str, offset: usize) -> usize {
        let counted = self.counted;
        let positions = content.chars().scan(0, |count, character| {
            Some((advance(count, character, counted), character))
        });
        let taken = positions.take_while(|(before, _)| *before < offset);
        taken.map(|(_, character)| (self.produced)(character)).sum()
    }
}
