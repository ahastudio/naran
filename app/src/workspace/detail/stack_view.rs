//! The stacked diffs on screen: a list that builds only the rows in view,
//! with the name of the file at the top kept in sight while it scrolls.

use std::ops::Range;

use gpui::{
    AnyElement, Context, Div, FontWeight, Hsla, ListHorizontalSizingBehavior, UniformList, div,
    prelude::*, px, uniform_list,
};

use super::super::Workspace;
use super::stack::{StackRow, Stacked};
use super::truncating;
use crate::model::{self, LineKind};
use crate::theme::Theme;
use crate::widgets::{muted, problem, status_badge};

/// Height of every row of the stacked diff.
const STACK_ROW: f32 = 20.;

impl Workspace {
    /// The picked files' diffs, one after another, in a list that builds only
    /// the rows in view.
    pub(super) fn render_stacked(
        &self,
        stacked: &Stacked,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match stacked_content(stacked, theme) {
            Ok(rows) => self.render_stack_list(rows, theme, cx),
            Err(shown) => shown,
        }
    }

    fn render_stack_list(
        &self,
        rows: &[StackRow],
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *theme;
        let list = self.stack_list(rows, theme, cx);
        let scrolled = -f32::from(self.scrolls.stacked.0.borrow().base_handle.offset().y);
        let scroll = StackScroll {
            scrolled,
            row: STACK_ROW,
        };
        div()
            .relative()
            .flex_1()
            .min_h(px(0.))
            .child(list)
            .children(sticky_file_head(rows, scroll, &theme))
            .into_any_element()
    }

    fn stack_list(&self, rows: &[StackRow], theme: Theme, cx: &mut Context<Self>) -> UniformList {
        let rows_in_view = cx.processor(move |this, range: Range<usize>, _window, _cx| {
            this.stack_rows(range, &theme)
        });
        uniform_list("stacked-diff", rows.len(), rows_in_view)
            .with_width_from_item(widest(rows))
            .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
            .track_scroll(self.scrolls.stacked.clone())
            .size_full()
    }

    pub(super) fn stack_rows(&self, range: Range<usize>, theme: &Theme) -> Vec<AnyElement> {
        let stacked = self.shown.stacked.as_ref();
        let Some(Ok(rows)) = stacked.and_then(|stacked| stacked.rows.as_ref()) else {
            return Vec::new();
        };
        let sideways = -f32::from(self.scrolls.stacked.0.borrow().base_handle.offset().x);
        range
            .filter_map(|index| rows.get(index))
            .map(|row| stack_row_element(row, sideways, theme))
            .collect()
    }
}

/// The rows of the stacked diff, or the element that says why there are none.
fn stacked_content<'a>(stacked: &'a Stacked, theme: &Theme) -> Result<&'a [StackRow], AnyElement> {
    match &stacked.rows {
        None => Err(muted("Reading…", theme)),
        Some(Err(error)) => Err(problem(error.to_string(), theme)),
        Some(Ok(rows)) => Ok(rows),
    }
}

/// How far the stacked diffs are scrolled down, in pixels, and how tall each
/// of their rows is.
#[derive(Debug, Clone, Copy)]
pub(super) struct StackScroll {
    pub(super) scrolled: f32,
    pub(super) row: f32,
}

impl StackScroll {
    /// The row at the top edge, for a scroll that is not negative.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a scroll position is not negative and far below usize::MAX rows"
    )]
    fn first_row(self) -> usize {
        (self.scrolled / self.row) as usize
    }

    /// How far up the head is pushed by the next head coming in below the
    /// row `first`.
    fn push_shift(self, rows: &[StackRow], first: usize) -> f32 {
        if !matches!(rows.get(first + 1), Some(StackRow::File { .. })) {
            return 0.;
        }
        -(self.scrolled - f32::from(px(self.row) * first))
    }
}

/// Which file head stays at the top of the stacked diffs when they are
/// scrolled as `scroll` says, and how far up it is pushed by the next head
/// coming in. `None` at the very top, where the first head is in its own place.
pub(super) fn sticky_head(rows: &[StackRow], scroll: StackScroll) -> Option<(usize, f32)> {
    if scroll.scrolled <= 0. || rows.is_empty() {
        return None;
    }
    let first = scroll.first_row().min(rows.len() - 1);
    let head = rows[..=first]
        .iter()
        .rposition(|row| matches!(row, StackRow::File { .. }))?;
    Some((head, scroll.push_shift(rows, first)))
}

/// The name of the file at the top stays there until the next file's name
/// pushes it out.
fn sticky_file_head(rows: &[StackRow], scroll: StackScroll, theme: &Theme) -> Option<AnyElement> {
    let (index, shift) = sticky_head(rows, scroll)?;
    let head = rows[index].file();
    head.map(|(path, badge)| {
        file_head(path, badge, theme)
            .debug_selector(|| format!("sticky-head-{index}"))
            .absolute()
            .top(px(shift))
            .left_0()
            .right_0()
            .into_any_element()
    })
}

impl StackRow {
    /// The path and badge of a file head.
    pub(super) fn file(&self) -> Option<(&str, char)> {
        match self {
            Self::File { path, badge } => Some((path, *badge)),
            _ => None,
        }
    }
}

/// `sideways` is how far the list is scrolled to the right: a file head moves
/// back by that much, so its name stays where it is.
fn stack_row_element(row: &StackRow, sideways: f32, theme: &Theme) -> AnyElement {
    match row {
        StackRow::File { path, badge } => file_head(path, *badge, theme)
            .relative()
            .left(px(sideways))
            .into_any_element(),
        StackRow::Note(text) => note_row(text, theme),
        StackRow::Line(line) => line_row(line, theme),
    }
}

/// The head of one file in the stacked diffs: its state and its path.
fn file_head(path: &str, badge: char, theme: &Theme) -> Div {
    let name = truncating()
        .font_weight(FontWeight::SEMIBOLD)
        .child(path.to_string());
    stack_row_frame()
        .gap_2()
        .bg(theme.surface)
        .child(status_badge(badge, theme))
        .child(name)
}

/// A row of the stacked diffs, all of one height.
fn stack_row_frame() -> Div {
    div().w_full().h(px(STACK_ROW)).flex().items_center().px_2()
}

fn note_row(text: &str, theme: &Theme) -> AnyElement {
    stack_row_frame()
        .text_color(theme.muted)
        .child(text.to_string())
        .into_any_element()
}

fn line_row(line: &str, theme: &Theme) -> AnyElement {
    let kind = model::classify(line);
    stack_row_frame()
        .bg(kind_background(kind, theme))
        .text_color(kind_color(kind, theme))
        .font_family("Menlo")
        .text_size(px(12.))
        .whitespace_nowrap()
        .child(visible_text(line))
        .into_any_element()
}

pub(super) fn kind_background(kind: LineKind, theme: &Theme) -> Hsla {
    match kind {
        LineKind::Added => theme.added.opacity(0.14),
        LineKind::Removed => theme.removed.opacity(0.14),
        LineKind::Hunk => theme.surface.opacity(0.6),
        LineKind::Meta | LineKind::Context => gpui::transparent_black(),
    }
}

pub(super) fn kind_color(kind: LineKind, theme: &Theme) -> Hsla {
    match kind {
        LineKind::Hunk | LineKind::Meta => theme.muted,
        LineKind::Added | LineKind::Removed | LineKind::Context => theme.text,
    }
}

/// An empty line keeps its height; tabs are shown as four spaces.
pub(super) fn visible_text(line: &str) -> String {
    if line.is_empty() {
        return " ".to_string();
    }
    line.replace('\t', "    ")
}

/// The row of the stacked diffs with the longest text, which sets the width
/// of every row.
pub(super) fn widest(rows: &[StackRow]) -> Option<usize> {
    rows.iter()
        .enumerate()
        .max_by_key(|(_, row)| match row {
            StackRow::File { path, .. } => columns(path),
            StackRow::Note(text) => columns(text),
            StackRow::Line(line) => columns(&line.replace('\t', "    ")),
        })
        .map(|(index, _)| index)
}

/// How many monospace columns a text takes: letters outside ASCII, such as
/// Korean, are drawn about twice as wide.
pub(super) fn columns(text: &str) -> usize {
    text.chars()
        .map(|letter| if letter.is_ascii() { 1 } else { 2 })
        .sum()
}
