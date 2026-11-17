//! The diff pane: the picked commit's open file, the stacked diffs of the
//! picked files, or the open file of the working lists with lines, hunks and
//! links to stage or unstage them.

use std::cell::RefCell;
use std::ops::Range;

use gpui::{
    AnyElement, Context, Div, FontWeight, Hsla, SharedString, Stateful, div, prelude::*, px,
};
use naran_core::{Error, Staging};

use super::super::Workspace;
use super::diff_rows::{
    DIFF_LIMIT, DiffLook, diff_body_base, named_lines, number_width, omitted_note,
};
use super::pictures::{image_format, render_pictures};
use super::staging::stage_label;
use super::truncating;
use super::viewing::Viewing;
use crate::model::{self, DiffRow, LineKind};
use crate::selectable::{Which, area};
use crate::theme::Theme;
use crate::widgets::{link, muted, problem};

/// What the rows of a diff are decorated from.
struct DiffView<'a> {
    viewing: &'a Viewing,
    lines: &'a [String],
    hunks: Vec<Range<usize>>,
    theme: &'a Theme,
}

impl DiffView<'_> {
    /// Added and removed lines can be picked when the diff is of a file with changes to split.
    fn pickable(&self, kind: LineKind) -> bool {
        self.viewing.partial() && matches!(kind, LineKind::Added | LineKind::Removed)
    }
}

impl Workspace {
    pub(super) fn render_diff(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let stacked = self.shown.stacked.as_ref();
        match (self.shown.inspecting.as_ref(), stacked) {
            (_, Some(stacked)) => fixed_pane().child(self.render_stacked(stacked, theme, cx)),
            (Some(inspecting), None) => {
                diff_pane().child(self.render_commit_diff(inspecting, theme))
            }
            (None, None) => self.render_viewing(theme, cx),
        }
    }

    fn render_viewing(&self, theme: &Theme, cx: &mut Context<Self>) -> Stateful<Div> {
        let Some(viewing) = self.shown.viewing.as_ref() else {
            return diff_pane().child(muted("Select a file to see its changes.", theme));
        };
        match viewing_content(viewing, theme) {
            Ok(lines) => self.render_lines(viewing, lines, theme, cx),
            Err(shown) => diff_pane().child(shown),
        }
    }

    fn render_lines(
        &self,
        viewing: &Viewing,
        lines: &[String],
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let header = diff_header(viewing, theme, cx);
        let view = DiffView {
            viewing,
            lines,
            hunks: model::hunks(lines),
            theme,
        };
        let body = self.diff_body(&view, cx);
        // A diff of text scrolls below its fixed name row, so this pane
        // itself does not scroll; the stacked diffs scroll on their own too.
        fixed_pane().child(named_lines(
            header.flex_none().debug_selector(|| "diff-name".into()),
            area(body, &self.texts, Which::Diff),
            &self.scrolls.diff_sideways,
        ))
    }

    fn diff_body(&self, view: &DiffView, cx: &mut Context<Self>) -> Div {
        let lines = view.lines;
        let rows = model::diff_rows(&lines[..lines.len().min(DIFF_LIMIT)]);
        let (body, texts) = self.body_rows(view, &rows, cx);
        self.texts.diff.set_lines(texts);
        omitted_note(body, lines.len(), view.theme)
    }

    fn body_rows(
        &self,
        view: &DiffView,
        rows: &[DiffRow],
        cx: &mut Context<Self>,
    ) -> (Div, Vec<SharedString>) {
        let look = DiffLook {
            id: "diff-line",
            theme: view.theme,
            region: &self.texts.diff,
            number_width: number_width(rows),
            texts: RefCell::default(),
        };
        let body = rows.iter().fold(diff_body_base(), |body, row| {
            body.child(self.decorate_row(look.row(row), row, view, cx))
        });
        (body, look.take_texts())
    }

    fn decorate_row(
        &self,
        element: Stateful<Div>,
        row: &DiffRow,
        view: &DiffView,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let index = row.index();
        match row {
            DiffRow::Line { kind, .. } => self.decorate_line(element, (index, *kind), view, cx),
            DiffRow::Hunk { .. } if view.viewing.partial() => hunk_link(element, index, view, cx),
            _ => element,
        }
    }

    /// A picked line is tinted; a line that can be picked is picked by a click.
    fn decorate_line(
        &self,
        element: Stateful<Div>,
        (index, kind): (usize, LineKind),
        view: &DiffView,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let picked = view.viewing.picked.contains(&index);
        let element = element.when(picked, |line| line.bg(view.theme.accent.opacity(0.35)));
        if !view.pickable(kind) {
            return element;
        }
        self.pick_on_click(element, index, view.theme.hover, cx)
    }

    fn pick_on_click(
        &self,
        element: Stateful<Div>,
        index: usize,
        hover: Hsla,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let region = self.texts.diff.clone();
        element
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .on_click(cx.listener(move |this, _, _, cx| {
                // The end of a selection is not a click on the line.
                if !region.dragged() {
                    this.toggle_pick(index, cx);
                }
            }))
    }
}

/// The diff pane when it scrolls as a whole.
pub(super) fn diff_pane() -> Stateful<Div> {
    fixed_pane().overflow_y_scroll()
}

/// The diff pane when what is in it scrolls on its own.
pub(super) fn fixed_pane() -> Stateful<Div> {
    div()
        .id("diff")
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .flex()
        .flex_col()
}

/// The lines of the open file, or the element that says why there are none.
fn viewing_content<'a>(viewing: &'a Viewing, theme: &Theme) -> Result<&'a [String], AnyElement> {
    if image_format(&viewing.file).is_some() {
        let pictures = render_pictures(&viewing.file, viewing.pictures.as_ref(), theme);
        return Err(pictures.into_any_element());
    }
    lines_content(viewing.lines.as_ref(), theme)
}

/// The lines of a diff, or the element that says why there are none.
pub(super) fn lines_content<'a>(
    lines: Option<&'a Result<Vec<String>, Error>>,
    theme: &Theme,
) -> Result<&'a [String], AnyElement> {
    match lines {
        None => Err(muted("Reading…", theme)),
        Some(Err(error)) => Err(problem(error.to_string(), theme)),
        Some(Ok(lines)) if lines.is_empty() => Err(muted("No changes to show.", theme)),
        Some(Ok(lines)) => Ok(lines),
    }
}

fn diff_header(viewing: &Viewing, theme: &Theme, cx: &mut Context<Workspace>) -> Div {
    let header = diff_header_base(viewing, theme);
    if !viewing.partial() || viewing.picked.is_empty() {
        return header;
    }
    let picked: Vec<usize> = viewing.picked.iter().copied().collect();
    header.children(pick_links(picked, viewing.staging, theme, cx))
}

/// The links that apply the picked lines and clear the picks.
fn pick_links(
    picked: Vec<usize>,
    staging: Staging,
    theme: &Theme,
    cx: &mut Context<Workspace>,
) -> [Stateful<Div>; 2] {
    let label = format!("{} ({})", stage_label(staging, "lines"), picked.len());
    let apply = link("apply-lines", label, theme)
        .on_click(cx.listener(move |this, _, _, cx| this.apply_lines(picked.clone(), cx)));
    let clear = link("clear-lines", "Clear", theme)
        .text_color(theme.muted)
        .on_click(cx.listener(|this, _, _, cx| this.clear_picks(cx)));
    [apply, clear]
}

fn diff_header_base(viewing: &Viewing, theme: &Theme) -> Div {
    let name = truncating()
        .font_weight(FontWeight::SEMIBOLD)
        .child(viewing.file.clone());
    div()
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .bg(theme.surface)
        .child(name)
}

fn hunk_link(
    element: Stateful<Div>,
    index: usize,
    view: &DiffView,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let hunk = view.hunks.iter().find(|range| range.start == index);
    element.children(hunk.map(|range| hunk_apply_link(range, view, cx)))
}

fn hunk_apply_link(
    range: &Range<usize>,
    view: &DiffView,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let in_hunk = model::changed_lines(view.lines, range.clone());
    let label = stage_label(view.viewing.staging, "hunk");
    link(("hunk", range.start), label, view.theme)
        .on_click(cx.listener(move |this, _, _, cx| this.apply_lines(in_hunk.clone(), cx)))
}
