//! The right column as a whole: the commit list on top, and below it the
//! files on the left and the diff on the right, split by dividers.

use gpui::{Context, Div, MouseButton, Stateful, div, prelude::*, px};

use super::super::Workspace;
use super::super::layout::Divider;
use super::super::state::{Pane, Pressed};
use super::loaded::Detail;
use crate::theme::Theme;
use crate::widgets::{identified, muted};

/// The right column: the commit list on top, files and diff below.
fn detail_column() -> Div {
    div()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .flex()
        .flex_col()
        .overflow_hidden()
}

impl Workspace {
    pub(in crate::workspace) fn render_detail(
        &self,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let column = detail_column();
        match self.selected_detail() {
            Some(detail) => column
                .child(self.commits_pane(detail, theme, cx))
                .child(self.divider(Divider::Commits, theme, cx))
                .child(self.lower_pane(detail, theme, cx)),
            None => column.child(muted(self.waiting_text(), theme)),
        }
    }

    /// The detail of the selected repository, once it is read.
    fn selected_detail(&self) -> Option<&Detail> {
        let selected = self.selection.repository.as_ref();
        self.shown
            .detail
            .as_ref()
            .filter(|detail| Some(&detail.path) == selected)
    }

    fn waiting_text(&self) -> &'static str {
        if self.selection.repository.is_some() {
            "Reading…"
        } else {
            "Select a repository."
        }
    }

    fn commits_pane(
        &self,
        detail: &Detail,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        identified("commits")
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.focus_pane(Pane::Commits)),
            )
            .h(px(self.layout.sizes.commits))
            .flex_none()
            .overflow_hidden()
            .child(self.render_history(detail, theme, cx))
    }

    /// The files on the left, the diff on the right.
    fn lower_pane(&self, detail: &Detail, theme: &Theme, cx: &mut Context<Self>) -> Div {
        div()
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_row()
            .overflow_hidden()
            .child(self.render_files(detail, theme, cx))
            .child(self.divider(Divider::Files, theme, cx))
            .child(self.diff_column(theme, cx))
    }

    fn diff_column(&self, theme: &Theme, cx: &mut Context<Self>) -> Div {
        div()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .flex()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.interaction.pressed = Pressed::Text),
            )
            .child(self.render_diff(theme, cx))
    }

    /// The column the files are listed in, the commit's or the working ones.
    pub(super) fn files_column(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        // The list is focused first (capture), so a press on the commit
        // information below can then mark its text as the last one pressed.
        div()
            .id("files")
            .capture_any_mouse_down(cx.listener(|this, _, _, _| this.focus_pane(Pane::Files)))
            .w(px(self.layout.sizes.files))
            .flex_none()
            .h_full()
            .overflow_hidden()
            .flex()
            .flex_col()
    }
}
