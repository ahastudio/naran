//! The element that shapes and paints the text, the selection and the cursor.

use std::ops::Range;

use gpui::{
    App, Bounds, ElementId, ElementInputHandler, Entity, FocusHandle, Font, GlobalElementId, Hsla,
    LayoutId, PaintQuad, Pixels, ShapedLine, SharedString, Style, TextRun, TextStyle,
    UnderlineStyle, Window, fill, point, prelude::*, px, relative, size,
};

use super::TextInput;

pub(super) struct TextElement {
    input: Entity<TextInput>,
}

impl TextElement {
    pub(super) fn new(input: Entity<TextInput>) -> Self {
        Self { input }
    }
}

pub(super) struct PrepaintState {
    line: ShapedLine,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let line = input.shape(window);
        PrepaintState {
            cursor: input.cursor_quad(&line, bounds),
            selection: input.selection_quad(&line, bounds),
            line,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        let line = prepaint.paint_all(bounds, &focus_handle, window, cx);
        self.input
            .update(cx, |input, _cx| input.remember_layout(line, bounds));
    }
}

impl PrepaintState {
    /// Paints the selection, the text and, when the input has focus, the
    /// cursor. Gives the text line back.
    fn paint_all(
        &mut self,
        bounds: Bounds<Pixels>,
        focus_handle: &FocusHandle,
        window: &mut Window,
        cx: &mut App,
    ) -> ShapedLine {
        paint_quad(self.selection.take(), window);
        let line = std::mem::take(&mut self.line);
        line.paint(bounds.origin, window.line_height(), window, cx)
            .ok();
        paint_quad(
            self.cursor
                .take()
                .filter(|_| focus_handle.is_focused(window)),
            window,
        );
        line
    }
}

fn paint_quad(quad: Option<PaintQuad>, window: &mut Window) {
    if let Some(quad) = quad {
        window.paint_quad(quad);
    }
}

impl TextInput {
    /// The text to show as one plain run: the placeholder, in its own color,
    /// while the input is empty.
    fn shown_text(&self, style: &TextStyle) -> (SharedString, TextRun) {
        let (text, color) = if self.content.is_empty() {
            (self.look.placeholder.clone(), self.look.placeholder_color)
        } else {
            (self.content.clone(), style.color)
        };
        let run = plain_run(text.len(), color, style.font());
        (text, run)
    }

    /// The line to paint: the text, or the placeholder while empty, with the
    /// composed part underlined.
    fn shape(&self, window: &Window) -> ShapedLine {
        let style = window.text_style();
        let (text, run) = self.shown_text(&style);
        let runs = text_runs(run, self.marked_range.as_ref());
        let font_size = style.font_size.to_pixels(window.rem_size());
        window
            .text_system()
            .shape_line(text, font_size, &runs, None)
    }

    fn cursor_quad(&self, line: &ShapedLine, bounds: Bounds<Pixels>) -> Option<PaintQuad> {
        let left = bounds.left() + line.x_for_index(self.cursor_offset());
        let caret = Bounds::new(
            point(left, bounds.top()),
            size(px(1.5), bounds.bottom() - bounds.top()),
        );
        self.selected_range
            .is_empty()
            .then(|| fill(caret, self.look.cursor_color))
    }

    fn selection_quad(&self, line: &ShapedLine, bounds: Bounds<Pixels>) -> Option<PaintQuad> {
        let (start, end) = (self.selected_range.start, self.selected_range.end);
        let shaded = Bounds::from_corners(
            point(bounds.left() + line.x_for_index(start), bounds.top()),
            point(bounds.left() + line.x_for_index(end), bounds.bottom()),
        );
        (!self.selected_range.is_empty())
            .then(|| fill(shaded, self.look.cursor_color.opacity(0.25)))
    }
}

/// The text as one run of one font and color, without decoration.
pub(super) fn plain_run(length: usize, color: Hsla, font: Font) -> TextRun {
    TextRun {
        len: length,
        font,
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    }
}

/// The text as one run, or as three when part of it is being composed (IME),
/// which gets an underline.
pub(super) fn text_runs(run: TextRun, marked: Option<&Range<usize>>) -> Vec<TextRun> {
    match marked {
        Some(marked) => composed_runs(&run, marked),
        None => vec![run],
    }
}

/// Before, inside and after the composed part; empty runs are left out.
fn composed_runs(run: &TextRun, marked: &Range<usize>) -> Vec<TextRun> {
    let underline = UnderlineStyle {
        color: Some(run.color),
        thickness: px(1.0),
        wavy: false,
    };
    let lengths = [marked.start, marked.len(), run.len - marked.end];
    let underlines = [None, Some(underline), None];
    let runs = lengths
        .into_iter()
        .zip(underlines)
        .map(|(length, underline)| TextRun {
            len: length,
            underline,
            ..run.clone()
        });
    runs.filter(|run| run.len > 0).collect()
}
