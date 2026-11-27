//! The worktree row: the facts of a repository row, indented under the
//! repository it belongs to.

use gpui::{Div, SharedString, Stateful, div, prelude::*, px};

use super::{identified, problem_marker, row_details, title_line};
use crate::model::Entry;
use crate::theme::Theme;

/// How far the branch mark of a worktree row indents its text.
const BRANCH_MARK: f32 = 18.;

/// One worktree below its repository. Identified by its folder, because two
/// repositories can each have a worktree of the same name. A branch mark in
/// a branch mark and a tinted background tie it to the row above.
pub fn worktree_row(entry: &Entry, theme: &Theme) -> Stateful<Div> {
    let hover = theme.hover;
    identified(SharedString::from(format!(
        "worktree-row-{}",
        entry.path().display()
    )))
    .flex()
    .items_center()
    .bg(theme.surface)
    .border_l_2()
    .border_color(problem_marker(entry, theme))
    .cursor_pointer()
    .hover(move |style| style.bg(hover))
    .child(branch_mark(theme))
    .child(worktree_text(entry, theme))
}

/// The column that holds the mark.
fn branch_mark(theme: &Theme) -> Div {
    div()
        .w(px(BRANCH_MARK))
        .flex_none()
        .flex()
        .justify_center()
        .text_color(theme.muted)
        .child("└")
}

fn worktree_text(entry: &Entry, theme: &Theme) -> Div {
    div()
        .flex_1()
        .min_w(px(0.))
        .flex()
        .flex_col()
        .gap_0p5()
        .pl_2()
        .pr_3()
        .py_1p5()
        .child(title_line(entry, entry.folder(), theme))
        .child(row_details(entry, theme))
}

/// Whether the worktrees of a repository are shown below it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Fold {
    Open,
    Folded,
}

/// The toggle at the right end of a repository row: an arrow and the number
/// of worktrees. A repository without worktrees has none.
pub fn worktree_slot(entry: &Entry, fold: Fold, theme: &Theme) -> Stateful<Div> {
    let arrow = if fold == Fold::Open { "▾" } else { "▸" };
    let count = entry.worktree_count().to_string();
    identified(SharedString::from(format!(
        "worktree-toggle-{}",
        entry.path().display()
    )))
    .flex_none()
    .flex()
    .items_center()
    .gap_0p5()
    .px_2()
    .text_size(px(12.))
    .text_color(theme.muted)
    .cursor_pointer()
    .child(arrow)
    .child(count)
}

/// The bar above the detail of a worktree: the repository it belongs to, its
/// folder and its branch, so the changes below are not taken for the
/// repository's own.
pub fn worktree_banner(repository: &str, worktree: &Entry, theme: &Theme) -> Stateful<Div> {
    identified("worktree-banner")
        .flex_none()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_1()
        .bg(theme.surface)
        .border_b_1()
        .border_color(theme.border)
        .text_size(px(12.))
        .text_color(theme.muted)
        .child(format!("Worktree of {repository}"))
        .child(
            div()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.text)
                .child(worktree.folder()),
        )
        .child(branch_label(worktree, theme))
}

fn branch_label(worktree: &Entry, theme: &Theme) -> Div {
    let branch = match worktree {
        Entry::Ready(summary) => summary.status.branch.clone(),
        Entry::Failed { .. } => None,
    };
    div()
        .text_color(theme.accent)
        .child(branch.unwrap_or_default())
}
