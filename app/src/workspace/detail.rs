//! The right column, laid out like Sourcetree: the commit list on top, and
//! below it the staged and unstaged files on the left with the diff of the
//! open file on the right. The only operations are staging and unstaging,
//! by file, by hunk and by line.
//!
//! The column is part of the workspace entity. Its child modules each hold
//! one part of it: what is read and kept (`loaded`, `commit`, `viewing`,
//! `stack`), what the keys and clicks do (`history`, `inspecting`,
//! `picking`, `stepping`, `staging`), and how each pane is drawn (the
//! `*_view`, `*_rows` and `*_lists` modules, `columns` and `painting`).

use gpui::{Div, ScrollStrategy, SharedString, div, prelude::*, px};

mod columns;
mod commit;
mod commit_info;
mod commit_picking;
mod commit_view;
mod diff_rows;
mod diff_view;
mod file_lists;
mod file_rows;
mod history;
mod history_rows;
mod history_view;
mod inspecting;
mod lanes;
mod loaded;
mod marks;
mod painting;
mod picking;
mod pictures;
mod stack;
mod stack_view;
mod staging;
mod stepping;
#[cfg(test)]
mod tests;
mod versions;
mod viewing;

pub(super) use commit::Inspecting;
pub(super) use loaded::Detail;
#[cfg(test)]
pub(super) use marks::ListedFile;
pub(super) use marks::Marked;
#[cfg(test)]
pub(super) use stack::StackRow;
pub(super) use stack::Stacked;
pub(super) use viewing::Viewing;

/// Height of a row of the commit list; the graph is drawn in this height.
const HISTORY_ROW: f32 = 24.;

/// Text that fills the rest of a row and is cut when it does not fit.
fn truncated(text: impl Into<SharedString>) -> Div {
    truncating().child(text.into())
}

/// A box that fills the rest of a row and cuts the text that does not fit.
pub(super) fn truncating() -> Div {
    div().flex_1().min_w(px(0.)).truncate()
}

/// Where a row that is scrolled into view goes: at the bottom edge when moving
/// down, at the top edge when moving up. A row already in view stays where it is.
fn follow(delta: isize) -> ScrollStrategy {
    if delta > 0 {
        ScrollStrategy::Bottom
    } else {
        ScrollStrategy::Top
    }
}
