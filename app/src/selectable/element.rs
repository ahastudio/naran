//! A line of selectable text as a GPUI element: it shapes the text, paints
//! the selected part behind it, and tells its region where it was painted.

use std::ops::Range;

use gpui::{
    App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId,
    IntoElement, LayoutId, Pixels, SharedString, Size, Style, TextAlign, TextRun, Window,
    WrappedLine, fill, point, px,
};

use super::Region;
use super::region::Placed;

/// One line of selectable text. It wraps (the commit message) or stays on
/// one line (a diff). Font, size and color come from the parent.
pub struct SelectableText {
    region: Region,
    index: usize,
    text: SharedString,
    wrap: bool,
    highlight: Hsla,
}

impl SelectableText {
    /// A line that stays on one line. `index` is the place of `text` in the
    /// lines given to `Region::set_lines`.
    pub fn new(region: &Region, index: usize, text: impl Into<SharedString>) -> Self {
        Self {
            region: region.clone(),
            index,
            text: text.into(),
            wrap: false,
            highlight: gpui::transparent_black(),
        }
    }

    /// Wraps the line at the width it is given, like a commit message.
    #[must_use]
    pub fn wrapped(self) -> Self {
        Self { wrap: true, ..self }
    }

    /// The color behind the selected part. Without it the selection is not seen.
    #[must_use]
    pub fn highlighted(self, highlight: Hsla) -> Self {
        Self { highlight, ..self }
    }

    /// An empty line is still one line tall.
    fn shaped(&self) -> SharedString {
        if self.text.is_empty() {
            " ".into()
        } else {
            self.text.clone()
        }
    }
}

impl IntoElement for SelectableText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SelectableText {
    type RequestLayoutState = ();
    type PrepaintState = WrappedLine;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let shaping = Shaping::new(self, window);
        let layout = window
            .request_measured_layout(Style::default(), move |known, available, window, _cx| {
                shaping.measure(known, available, window)
            });
        (layout, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let wrap_width = self.wrap.then_some(bounds.size.width);
        Shaping::new(self, window).shape(window, wrap_width)
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let line = std::mem::take(prepaint);
        let line_height = window.line_height();
        self.paint_selection(&line, bounds, window);
        line.paint(
            bounds.origin,
            line_height,
            TextAlign::Left,
            Some(bounds),
            window,
            cx,
        )
        .ok();
        self.region.place(
            self.index,
            Placed {
                bounds,
                layout: line,
                line_height,
            },
        );
    }
}

impl SelectableText {
    fn paint_selection(&self, line: &WrappedLine, bounds: Bounds<Pixels>, window: &mut Window) {
        if let Some(range) = self.region.highlight(self.index) {
            let rows = Rows::new(line, bounds, window.line_height());
            rows.paint_selected(&range, self.highlight, window);
        }
    }
}

/// What it takes to shape the text of a line: the font, the size and the run.
struct Shaping {
    text: SharedString,
    run: TextRun,
    font_size: Pixels,
    line_height: Pixels,
    wrap: bool,
}

impl Shaping {
    fn new(element: &SelectableText, window: &Window) -> Self {
        let style = window.text_style();
        let text = element.shaped();
        Self {
            font_size: style.font_size.to_pixels(window.rem_size()),
            line_height: window.line_height(),
            run: style.to_run(text.len()),
            wrap: element.wrap,
            text,
        }
    }

    /// The size the text takes: no wider than the space, when it wraps.
    fn measure(
        &self,
        known: Size<Option<Pixels>>,
        available: Size<AvailableSpace>,
        window: &mut Window,
    ) -> Size<Pixels> {
        let space = known.width.or_else(|| definite(available.width));
        let line = self.shape(window, self.wrap.then_some(space).flatten());
        line.size(self.line_height)
    }

    /// GPUI shapes text into at least one line, even an empty one.
    fn shape(&self, window: &mut Window, wrap_width: Option<Pixels>) -> WrappedLine {
        let run = std::slice::from_ref(&self.run);
        let shaped = window.text_system().shape_text(
            self.text.clone(),
            self.font_size,
            run,
            wrap_width,
            None,
        );
        shaped
            .ok()
            .and_then(|mut lines| lines.pop())
            .unwrap_or_default()
    }
}

fn definite(width: AvailableSpace) -> Option<Pixels> {
    match width {
        AvailableSpace::Definite(width) => Some(width),
        _ => None,
    }
}

/// The rows a line is wrapped into.
struct Rows<'a> {
    line: &'a WrappedLine,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
    count: usize,
}

impl<'a> Rows<'a> {
    fn new(line: &'a WrappedLine, bounds: Bounds<Pixels>, line_height: Pixels) -> Self {
        let count = line.wrap_boundaries().len() + 1;
        Self {
            line,
            bounds,
            line_height,
            count,
        }
    }

    /// The byte at height `row` closest to `x`.
    fn closest(&self, x: Pixels, row: usize) -> usize {
        let inside = point(x, self.line_height * row + self.line_height * 0.5);
        self.line
            .closest_index_for_position(inside, self.line_height)
            .unwrap_or_else(|index| index)
    }

    /// Paints the selected bytes of the line, one rectangle for each row the
    /// line is wrapped into.
    fn paint_selected(&self, range: &Range<usize>, color: Hsla, window: &mut Window) {
        (0..self.count)
            .filter_map(|row| self.selected_quad(row, range))
            .for_each(|quad| window.paint_quad(fill(quad, color)));
    }

    /// The part of `range` inside `row`.
    fn selected_part(&self, row: usize, range: &Range<usize>) -> Option<RowPart> {
        let row_end = self.closest(px(100_000.), row);
        let from = range.start.max(self.closest(px(0.), row));
        let to = range.end.min(row_end);
        (from < to).then_some(RowPart {
            row,
            bytes: from..to,
            row_end,
        })
    }

    fn selected_quad(&self, row: usize, range: &Range<usize>) -> Option<Bounds<Pixels>> {
        let part = self.selected_part(row, range)?;
        let top = self.bounds.top() + self.line_height * row;
        let left = self.bounds.left() + self.left_edge(part.bytes.start);
        let right = self.bounds.left() + self.right_edge(&part);
        Some(Bounds::from_corners(
            point(left, top),
            point(right, top + self.line_height),
        ))
    }

    fn left_edge(&self, from: usize) -> Pixels {
        self.line
            .position_for_index(from, self.line_height)
            .map_or(px(0.), |position| position.x)
    }

    /// The end of a row that goes on in the next row is the right edge.
    fn right_edge(&self, part: &RowPart) -> Pixels {
        let goes_on = part.bytes.end >= part.row_end && part.row + 1 < self.count;
        if goes_on {
            return self.bounds.size.width;
        }
        self.line
            .position_for_index(part.bytes.end, self.line_height)
            .map_or(self.bounds.size.width, |position| position.x)
    }
}

/// The selected bytes inside one row of a wrapped line, and the byte where
/// that row ends.
struct RowPart {
    row: usize,
    bytes: Range<usize>,
    row_end: usize,
}
