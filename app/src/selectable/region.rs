//! The lines of one view, where they were painted, and the selection over them.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Range;
use std::rc::Rc;

use gpui::{Bounds, Pixels, Point, SharedString, WrappedLine, point, px};

/// A place in the text of a region: a line and a byte offset in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Spot {
    line: usize,
    offset: usize,
}

impl Spot {
    /// The same place, at most in `last_line`.
    fn within(self, last_line: usize) -> Self {
        Self {
            line: self.line.min(last_line),
            ..self
        }
    }
}

/// A selection that is not empty, from its first to its last place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    start: Spot,
    end: Spot,
}

impl Span {
    fn covers(self, line: usize) -> bool {
        (self.start.line..=self.end.line).contains(&line)
    }

    /// The selected bytes of the line that ends at `line_end`.
    fn bytes_in(self, line_end: Spot) -> Range<usize> {
        self.start_in(line_end)..self.end_in(line_end)
    }

    fn start_in(self, line_end: Spot) -> usize {
        if line_end.line == self.start.line {
            self.start.offset.min(line_end.offset)
        } else {
            0
        }
    }

    fn end_in(self, line_end: Spot) -> usize {
        if line_end.line == self.end.line {
            self.end.offset.min(line_end.offset)
        } else {
            line_end.offset
        }
    }
}

/// Where a line was painted last time.
pub(super) struct Placed {
    pub(super) bounds: Bounds<Pixels>,
    pub(super) layout: WrappedLine,
    pub(super) line_height: Pixels,
}

impl Placed {
    /// The byte of the line closest to the pointer.
    fn offset_at(&self, position: Point<Pixels>) -> usize {
        let bounds = self.bounds;
        let y = f32::from(position.y - bounds.top())
            .clamp(0., (f32::from(bounds.size.height) - 1.).max(0.));
        let inside = point(position.x - bounds.left(), px(y));
        let offset = self
            .layout
            .closest_index_for_position(inside, self.line_height)
            .unwrap_or_else(|index| index);
        offset.min(self.layout.text.len())
    }
}

#[derive(Default)]
struct Inner {
    lines: Vec<SharedString>,
    anchor: Option<Spot>,
    focus: Option<Spot>,
    dragging: bool,
    /// The pointer moved to another place since it was pressed.
    moved: bool,
    /// Set by `set_lines`, cleared by `begin_frame`: the region is on screen.
    shown: bool,
    placed: BTreeMap<usize, Placed>,
}

/// The lines of one view and the selection over them. Cloning gives another
/// handle to the same region.
#[derive(Clone, Default)]
pub struct Region(Rc<RefCell<Inner>>);

impl Region {
    /// Gives the text of the lines, in the order they are painted. A different
    /// text drops the selection.
    pub fn set_lines(&self, lines: Vec<SharedString>) {
        let mut inner = self.0.borrow_mut();
        inner.shown = true;
        inner.placed.clear();
        inner.replace_lines(lines);
    }

    /// A region that was not given lines since the last call is not on screen
    /// any more: it forgets its text and selection.
    pub(super) fn begin_frame(&self) {
        let mut inner = self.0.borrow_mut();
        if !inner.shown {
            inner.lines.clear();
            inner.clear_selection();
        }
        inner.shown = false;
    }

    pub(super) fn press(&self, position: Point<Pixels>) {
        let mut inner = self.0.borrow_mut();
        let spot = inner.spot_at(position);
        (inner.anchor, inner.focus) = (spot, spot);
        (inner.dragging, inner.moved) = (spot.is_some(), false);
    }

    /// Moves the end of the selection to the pointer. True when it changed.
    pub(super) fn drag(&self, position: Point<Pixels>) -> bool {
        self.0.borrow_mut().drag_to(position)
    }

    pub(super) fn release(&self) {
        let mut inner = self.0.borrow_mut();
        inner.dragging = false;
        // A click that did not move selects nothing.
        if !inner.moved {
            inner.clear_selection();
        }
    }

    /// The last press turned into a drag, so the click that follows its
    /// release is the end of a selection and not a click on a line.
    pub fn dragged(&self) -> bool {
        self.0.borrow().moved
    }

    pub fn clear(&self) {
        let mut inner = self.0.borrow_mut();
        inner.clear_selection();
    }

    pub(super) fn select_all(&self) {
        self.0.borrow_mut().select_everything();
    }

    /// The selected text, lines joined with a newline.
    pub(super) fn selected_text(&self) -> Option<String> {
        let inner = self.0.borrow();
        let span = inner.span().filter(|_| inner.shown)?;
        let lines: Option<Vec<&str>> = (span.start.line..=span.end.line)
            .map(|line| inner.selected_line(span, line))
            .collect();
        Some(lines?.join("\n"))
    }

    /// The selected bytes of a line, for painting.
    pub(super) fn highlight(&self, line: usize) -> Option<Range<usize>> {
        let inner = self.0.borrow();
        let span = inner.span().filter(|span| span.covers(line))?;
        let offset = inner.lines.get(line).map_or(0, |content| content.len());
        let range = span.bytes_in(Spot { line, offset });
        (range.start < range.end).then_some(range)
    }

    pub(super) fn place(&self, line: usize, placed: Placed) {
        self.0.borrow_mut().placed.insert(line, placed);
    }

    /// Where `line` was painted last, for tests that point at it.
    #[cfg(test)]
    pub(super) fn placed_bounds(&self, line: usize) -> Option<Bounds<Pixels>> {
        self.0
            .borrow()
            .placed
            .get(&line)
            .map(|placed| placed.bounds)
    }
}

impl Inner {
    fn clear_selection(&mut self) {
        self.anchor = None;
        self.focus = None;
        self.dragging = false;
        self.moved = false;
    }

    /// The selected part of one line of `span`.
    fn selected_line(&self, span: Span, line: usize) -> Option<&str> {
        let line_end = |content: &SharedString| Spot {
            line,
            offset: content.len(),
        };
        let content = self.lines.get(line);
        content.and_then(|content| content.get(span.bytes_in(line_end(content))))
    }

    /// The selection from its first to its last place, when it is not empty.
    fn span(&self) -> Option<Span> {
        let (anchor, focus) = (self.anchor?, self.focus?);
        let last_line = self.lines.len().checked_sub(1)?;
        let (start, end) = (anchor.min(focus), anchor.max(focus));
        (start != end).then(|| Span {
            start,
            end: end.within(last_line),
        })
    }

    /// A different text drops the selection.
    fn replace_lines(&mut self, lines: Vec<SharedString>) {
        if self.lines != lines {
            self.lines = lines;
            self.clear_selection();
        }
    }

    fn select_everything(&mut self) {
        let Some(last) = self.lines.len().checked_sub(1) else {
            return;
        };
        let end = Spot {
            line: last,
            offset: self.lines[last].len(),
        };
        (self.anchor, self.focus) = (Some(Spot { line: 0, offset: 0 }), Some(end));
        (self.dragging, self.moved) = (false, true);
    }

    /// Moves the end of the selection to the pointer. True when it changed.
    fn drag_to(&mut self, position: Point<Pixels>) -> bool {
        let Some(spot) = self.dragging.then(|| self.spot_at(position)) else {
            return false;
        };
        let changed = spot != self.focus;
        (self.focus, self.moved) = (spot, self.moved || changed);
        changed
    }

    /// The place of the text under `position`: the closest painted line, and
    /// the character boundary closest to the pointer in it.
    fn spot_at(&self, position: Point<Pixels>) -> Option<Spot> {
        let gap = |placed: &Placed| vertical_gap(position.y, &placed.bounds);
        let (line, placed) = self
            .placed
            .iter()
            .min_by(|(_, a), (_, b)| gap(a).total_cmp(&gap(b)))?;
        Some(Spot {
            line: *line,
            offset: placed.offset_at(position),
        })
    }
}

/// How far `y` is above or below `bounds`; zero inside.
fn vertical_gap(y: Pixels, bounds: &Bounds<Pixels>) -> f32 {
    let (above, below) = (f32::from(bounds.top() - y), f32::from(y - bounds.bottom()));
    above.max(below).max(0.)
}

#[cfg(test)]
mod tests;
