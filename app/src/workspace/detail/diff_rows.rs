//! The rows of a diff of text, Sourcetree style, shared by the open file of
//! the working lists and the open file of a picked commit.

use std::cell::RefCell;

use gpui::{Div, Hsla, Pixels, ScrollHandle, SharedString, Stateful, div, prelude::*, px};

use super::truncated;
use crate::model::{DiffRow, LineKind};
use crate::selectable::{Region, SelectableText};
use crate::theme::Theme;

/// Diffs longer than this are cut, so a huge file cannot freeze the window.
pub(super) const DIFF_LIMIT: usize = 3000;

/// What the rows of a diff share: colors, the width of the line numbers, and
/// the selectable text of the rows built so far.
pub(super) struct DiffLook<'a> {
    /// The id every row of this diff is named with, with its index.
    pub(super) id: &'static str,
    pub(super) theme: &'a Theme,
    pub(super) region: &'a Region,
    pub(super) number_width: Pixels,
    pub(super) texts: RefCell<Vec<SharedString>>,
}

/// Wide enough for the longest line number of `rows`, and at least two digits.
pub(super) fn number_width(rows: &[DiffRow]) -> Pixels {
    let most = rows.iter().map(line_number).max().unwrap_or(0);
    let digits = most.to_string().len().max(2);
    px(7.4) * digits + px(14.)
}

impl DiffLook<'_> {
    /// One row of a diff: the old and the new line number, a `+` or `-`,
    /// then the text. A hunk is a bar with its title. The text is the only
    /// part that can be selected.
    pub(super) fn row(&self, row: &DiffRow) -> Stateful<Div> {
        match row {
            DiffRow::Hunk { index, title } => self.hunk_row(*index, title),
            DiffRow::Note { index, text } => self.note_row(*index, text),
            DiffRow::Line {
                index,
                kind,
                old,
                new,
                text,
            } => self.line_row(*index, (*kind, *old, *new), text),
        }
    }

    /// The selectable text of the rows built so far, in their order.
    pub(super) fn take_texts(&self) -> Vec<SharedString> {
        self.texts.take()
    }

    fn hunk_row(&self, index: usize, title: &str) -> Stateful<Div> {
        let theme = self.theme;
        div()
            .id((self.id, index))
            .flex()
            .items_center()
            .w_full()
            .px_3()
            .py_1()
            .mt_1()
            .bg(theme.surface)
            .border_y_1()
            .border_color(theme.border)
            .text_size(px(11.))
            .text_color(theme.muted)
            .child(truncated(title.to_string()))
    }

    fn note_row(&self, index: usize, text: &str) -> Stateful<Div> {
        let at = self.push(text);
        div()
            .id((self.id, index))
            .w_full()
            .pl(self.number_width * 2. + px(18.))
            .py_0p5()
            .text_color(self.theme.muted)
            .child(self.selectable(at, text))
    }

    fn line_row(
        &self,
        index: usize,
        (kind, old, new): (LineKind, Option<usize>, Option<usize>),
        text: &str,
    ) -> Stateful<Div> {
        let (tint, marker) = kind_tint(kind, self.theme);
        let at = self.push(text);
        let numbers = [self.number(old, tint), self.number(new, tint)];
        let id = self.id;
        div()
            .id((id, index))
            .debug_selector(move || format!("{id}-{index}"))
            .flex()
            .w_full()
            .bg(tint.map_or(gpui::transparent_black(), |color| color.opacity(0.13)))
            .children(numbers)
            .child(marker_cell(marker, tint, self.theme))
            .child(self.line_text(at, text))
    }

    fn number(&self, value: Option<usize>, tint: Option<Hsla>) -> Div {
        div()
            .flex_none()
            .w(self.number_width)
            .px_1()
            .text_right()
            .text_color(self.theme.muted)
            .bg(tint.map_or(self.theme.surface, |color| color.opacity(0.22)))
            .child(value.map(|value| value.to_string()).unwrap_or_default())
    }

    /// Never cut: a long line makes the diff wider and it scrolls sideways.
    fn line_text(&self, at: usize, text: &str) -> Div {
        div()
            .flex_grow()
            .flex_shrink_0()
            .text_color(self.theme.text)
            .child(self.selectable(at, text))
    }

    fn selectable(&self, at: usize, text: &str) -> SelectableText {
        let highlight = self.theme.accent.opacity(0.35);
        SelectableText::new(self.region, at, text.to_string()).highlighted(highlight)
    }

    /// Adds the text of a row to the selectable ones and tells its place.
    fn push(&self, text: &str) -> usize {
        let mut texts = self.texts.borrow_mut();
        texts.push(text.to_string().into());
        texts.len() - 1
    }
}

fn line_number(row: &DiffRow) -> usize {
    match row {
        DiffRow::Line { old, new, .. } => (*old).max(*new).unwrap_or(0),
        _ => 0,
    }
}

fn kind_tint(kind: LineKind, theme: &Theme) -> (Option<Hsla>, &'static str) {
    match kind {
        LineKind::Added => (Some(theme.added), "+"),
        LineKind::Removed => (Some(theme.removed), "-"),
        _ => (None, " "),
    }
}

fn marker_cell(marker: &'static str, tint: Option<Hsla>, theme: &Theme) -> Div {
    div()
        .flex_none()
        .w(px(18.))
        .text_center()
        .text_color(tint.unwrap_or(theme.muted))
        .child(marker)
}

pub(super) fn diff_body_base() -> Div {
    div()
        .flex()
        .flex_col()
        .pb_1()
        .font_family("Menlo")
        .text_size(px(12.))
}

/// Every row of a diff, read only.
pub(super) fn plain_rows(rows: &[DiffRow], look: &DiffLook) -> Div {
    rows.iter()
        .fold(diff_body_base(), |body, row| body.child(look.row(row)))
}

/// A note under a diff that was cut at [`DIFF_LIMIT`] lines.
pub(super) fn omitted_note(body: Div, total: usize, theme: &Theme) -> Div {
    body.when(total > DIFF_LIMIT, |body| {
        body.child(
            div()
                .debug_selector(|| "omitted-note".into())
                .px_2()
                .text_color(theme.muted)
                .child(format!("… {} more lines.", total - DIFF_LIMIT)),
        )
    })
}

/// A view with a fixed name row above lines that scroll on their own, down
/// and sideways, so the name of the file stays in sight.
pub(super) fn named_lines(name: impl IntoElement, lines: Div, scroll: &ScrollHandle) -> Div {
    div().size_full().flex().flex_col().child(name).child(
        div()
            .id("diff-lines")
            .debug_selector(|| "diff-lines".into())
            .flex_1()
            .min_h(px(0.))
            .overflow_y_scroll()
            .child(sideways(lines, scroll)),
    )
}

/// Lets the lines of a diff run past the right edge and be scrolled to
/// sideways. The lines take the width of the longest one, so their colors
/// reach across, and at least the width of the pane.
fn sideways(lines: Div, scroll: &ScrollHandle) -> Stateful<Div> {
    div()
        .id("diff-sideways")
        .w_full()
        .flex()
        .overflow_x_scroll()
        .track_scroll(scroll)
        .child(lines.flex_none().min_w_full())
}
