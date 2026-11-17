//! The commit list: a head of column names, the rows built only while in
//! view, and a footer with Go to HEAD, Remote branches and All refs.

use std::ops::Range;

use gpui::{AnyElement, Context, Div, UniformList, div, prelude::*, px, uniform_list};
use naran_core::log::Commit;

use super::super::Workspace;
use super::history_rows::{HistoryView, history_cell};
use super::lanes::graph_width;
use super::loaded::Detail;
use super::truncating;
use crate::theme::Theme;
use crate::widgets::{button, muted, problem};

impl Workspace {
    pub(super) fn render_history(
        &self,
        detail: &Detail,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match history_content(detail, theme) {
            Ok(commits) => self.render_history_list(detail, commits, theme, cx),
            Err(shown) => shown,
        }
    }

    fn render_history_list(
        &self,
        detail: &Detail,
        commits: &[Commit],
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let total = commits.len() + detail.uncommitted_rows();
        let width = graph_width(&detail.rows, detail.history_link(commits));
        let list = self.history_list(total, cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(history_head(width, theme))
            .child(list)
            .child(self.history_footer(theme, cx))
            .into_any_element()
    }

    fn history_list(&self, total: usize, cx: &mut Context<Self>) -> UniformList {
        let rows = cx.processor(move |this, range: Range<usize>, _window, cx| {
            if range.end + 40 >= total {
                this.load_more_history(cx);
            }
            this.history_rows(range, cx)
        });
        uniform_list("history", total, rows)
            .track_scroll(self.scrolls.history.clone())
            .flex_1()
            .min_h(px(0.))
    }

    /// Go to HEAD, Remote branches and All refs live in a footer under the list, so they
    /// never cover rows.
    fn history_footer(&self, theme: &Theme, cx: &mut Context<Self>) -> Div {
        div()
            .flex_none()
            .flex()
            .items_center()
            .justify_end()
            .gap_2()
            .px_2()
            .py_1()
            .bg(theme.surface)
            .child(self.remote_branches_button(theme, cx))
            .child(self.all_refs_button(theme, cx))
            .child(
                button("goto-head", "Go to HEAD", theme)
                    .on_click(cx.listener(|this, _, _, cx| this.jump_to_head(cx))),
            )
    }

    fn all_refs_button(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        button("all-refs", all_refs_label(self.layout.refs), theme)
            .on_click(cx.listener(|this, _, _, cx| this.toggle_all_refs(cx)))
    }

    fn remote_branches_button(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        button(
            "remote-branches",
            remote_branches_label(self.layout.refs),
            theme,
        )
        .on_click(cx.listener(|this, _, _, cx| this.toggle_remote_branches(cx)))
    }

    /// The rows `range` of the list: the Uncommitted changes row (when there
    /// are changes) and then one row per commit, with its part of the graph.
    pub(super) fn history_rows(
        &self,
        range: Range<usize>,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let detail = self.shown.detail.as_ref();
        let Some((detail, Ok(commits))) = detail.map(|detail| (detail, detail.history.as_ref()))
        else {
            return Vec::new();
        };
        let view = HistoryView::new(detail, commits, self.shown.inspecting.as_ref());
        range.filter_map(|index| view.row(index, cx)).collect()
    }
}

/// The commits of the list, or the element that says why there are none.
fn history_content<'a>(detail: &'a Detail, theme: &Theme) -> Result<&'a [Commit], AnyElement> {
    match &detail.history {
        Err(error) => Err(problem(error.to_string(), theme)),
        Ok(commits) if commits.is_empty() => Err(muted("No commits yet.", theme)),
        Ok(commits) => Ok(commits),
    }
}

pub(super) fn all_refs_label(refs: naran_core::Refs) -> &'static str {
    if refs.everything {
        "All refs: on"
    } else {
        "All refs: off"
    }
}

pub(super) fn remote_branches_label(refs: naran_core::Refs) -> &'static str {
    if refs.remotes {
        "Remote branches: on"
    } else {
        "Remote branches: off"
    }
}

/// The column heads stay on one line however narrow the list gets.
fn history_head(graph_width: f32, theme: &Theme) -> Div {
    let graph = div()
        .flex_none()
        .w(px(graph_width))
        .text_color(theme.muted)
        .child("Graph");
    let description = truncating().text_color(theme.muted);
    history_head_frame(theme)
        .child(graph)
        .child(description.child("Description"))
        .child(history_cell(96., theme).child("Date"))
        .child(history_cell(140., theme).child("Author"))
        .child(history_cell(64., theme).child("Commit"))
}

/// The bar of column names above the commit list.
fn history_head_frame(theme: &Theme) -> Div {
    div()
        .debug_selector(|| "history-head".into())
        .flex_none()
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py_0p5()
        .whitespace_nowrap()
        .bg(theme.surface)
        .text_size(px(11.))
}
