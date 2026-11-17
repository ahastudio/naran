//! Small building blocks shared by the columns.

use gpui::{
    AnyElement, App, Context, Div, ElementId, Entity, FontWeight, Hsla, Render, SharedString,
    Stateful, Window, div, prelude::*, px,
};

use naran_core::Status;

use crate::model::{self, Entry, Tone};
use crate::theme::Theme;

mod badges;

pub use badges::{badge, head_badge, status_badge};

/// An element with an id, which tests can also find by that id.
pub fn identified(id: impl Into<ElementId>) -> Stateful<Div> {
    let id = id.into();
    let selector = id.to_string();
    div().id(id).debug_selector(|| selector)
}

/// A button that answers clicks.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    theme: &Theme,
) -> Stateful<Div> {
    let hover = theme.hover;
    button_frame(id, label, theme)
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
}

/// A button that is shown but faded, because it cannot be used right now.
pub fn disabled_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    theme: &Theme,
) -> Stateful<Div> {
    button_frame(id, label, theme)
        .text_color(theme.muted)
        .opacity(0.6)
}

fn button_frame(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    theme: &Theme,
) -> Stateful<Div> {
    identified(id)
        .flex_none()
        .px_2p5()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.background)
        .whitespace_nowrap()
        .child(label.into())
}

/// A borderless text button for actions next to titles and rows, in the
/// accent color unless the caller sets another.
pub fn link(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    theme: &Theme,
) -> Stateful<Div> {
    let hover = theme.hover;
    identified(id)
        .flex_none()
        .px_1p5()
        .py_0p5()
        .rounded_sm()
        .text_size(px(12.))
        .text_color(theme.accent)
        .whitespace_nowrap()
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .child(label.into())
}

pub fn section_title(text: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .px_1()
        .py_1()
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.muted)
        .whitespace_nowrap()
        .truncate()
        .child(text.into())
}

/// What follows the pointer while a row is dragged.
pub struct DragPreview {
    name: SharedString,
}

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::dark();
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(theme.accent)
            .bg(theme.surface)
            .text_color(theme.text)
            .child(self.name.clone())
    }
}

/// The preview of a dragged row, named after it.
pub fn drag_preview(name: &str, cx: &mut App) -> Entity<DragPreview> {
    let name = SharedString::from(name.to_owned());
    cx.new(|_| DragPreview { name })
}

/// A group as its row in the left column shows it.
pub struct GroupRow<'a> {
    pub name: SharedString,
    pub members: &'a [&'a Entry],
    pub selected: bool,
}

impl GroupRow<'_> {
    fn problems(&self) -> usize {
        let members = self.members.iter();
        members.filter(|entry| entry.has_problem()).count()
    }
}

pub fn group_row(id: impl Into<ElementId>, group: &GroupRow, theme: &Theme) -> Stateful<Div> {
    let problems = group.problems();
    let hover = theme.hover;
    identified(id)
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .child(div().flex_1().truncate().child(group.name.clone()))
        .child(muted_count(group.members.len(), theme))
        .when(problems > 0, |row| {
            row.child(badge(format!("!{problems}"), theme.problem))
        })
        .when(group.selected, |row| {
            row.bg(theme.selected).font_weight(FontWeight::SEMIBOLD)
        })
}

fn muted_count(count: usize, theme: &Theme) -> Div {
    div()
        .whitespace_nowrap()
        .text_color(theme.muted)
        .child(count.to_string())
}

/// One repository in the middle column: name and state on top, branch and
/// non-zero counts below.
pub fn repository_row(index: usize, entry: &Entry, theme: &Theme) -> Stateful<Div> {
    let hover = theme.hover;
    let marker = problem_marker(entry, theme);
    div()
        .id(("repository", index))
        .flex()
        .flex_col()
        .gap_0p5()
        .pl(px(10.))
        .pr_3()
        .py_2()
        .border_l_2()
        .border_color(marker)
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .child(title_line(entry, theme))
        .child(row_details(entry, theme))
}

/// A colored edge on the left of a repository with a problem.
fn problem_marker(entry: &Entry, theme: &Theme) -> Hsla {
    if entry.has_problem() {
        theme.problem
    } else {
        gpui::transparent_black()
    }
}

/// Name on the left, state on the right.
fn title_line(entry: &Entry, theme: &Theme) -> Div {
    let (state, tone) = model::describe(entry);
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex_1()
                .truncate()
                .font_weight(FontWeight::SEMIBOLD)
                .child(entry.name()),
        )
        .child(
            div()
                .flex_none()
                .whitespace_nowrap()
                .text_size(px(12.))
                .text_color(tone_color(tone, theme))
                .child(state),
        )
}

/// Branch and the counts that are not zero. A repository that could not be
/// read has no details.
fn row_details(entry: &Entry, theme: &Theme) -> Div {
    let details = div()
        .flex()
        .items_center()
        .gap_2()
        .text_size(px(12.))
        .text_color(theme.muted)
        .whitespace_nowrap()
        .overflow_hidden();
    match entry {
        Entry::Ready(summary) => details
            .child(branch_name(&summary.status, theme))
            .children(count_labels(&summary.status)),
        Entry::Failed { .. } => details,
    }
}

fn branch_name(status: &Status, theme: &Theme) -> Div {
    let branch = status.branch.clone();
    div()
        .text_color(theme.text)
        .child(branch.unwrap_or_else(|| "detached HEAD".into()))
}

fn count_labels(status: &Status) -> Vec<String> {
    let counts = [
        (status.ahead, "↑", ""),
        (status.behind, "↓", ""),
        (status.staged, "", " staged"),
        (status.unstaged, "", " modified"),
        (status.untracked, "", " untracked"),
        (status.conflicted, "", " conflicted"),
    ];
    counts
        .into_iter()
        .filter(|(count, ..)| *count > 0)
        .map(|(count, before, after)| format!("{before}{count}{after}"))
        .collect()
}

pub fn muted(text: impl Into<SharedString>, theme: &Theme) -> AnyElement {
    div()
        .p_2()
        .text_color(theme.muted)
        .child(text.into())
        .into_any_element()
}

pub fn problem(text: impl Into<SharedString>, theme: &Theme) -> AnyElement {
    div()
        .debug_selector(|| "problem".into())
        .p_2()
        .text_color(theme.problem)
        .child(text.into())
        .into_any_element()
}

pub fn tone_color(tone: Tone, theme: &Theme) -> Hsla {
    match tone {
        Tone::Problem => theme.problem,
        Tone::Attention => theme.attention,
        Tone::Calm => theme.calm,
    }
}

#[cfg(test)]
mod tests;
