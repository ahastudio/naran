//! The files column while no commit is picked: the Staged files list above
//! the Unstaged files list, each scrolling on its own.

use std::ops::Range;

use gpui::{
    AnyElement, Context, Div, SharedString, Stateful, UniformList, div, prelude::*, px,
    uniform_list,
};
use naran_core::{Change, Staging};

use super::super::Workspace;
use super::file_rows::{FileRow, file_row_button, file_row_frame, list_frame, side_name};
use super::loaded::Detail;
use crate::model::FileState;
use crate::theme::Theme;
use crate::widgets::{problem, section_title};

impl Workspace {
    pub(super) fn render_files(
        &self,
        detail: &Detail,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        if let Some(inspecting) = self.shown.inspecting.as_ref() {
            return self.render_commit_side(inspecting, theme, cx);
        }
        match &detail.changes {
            Ok(changes) => self.render_working_lists(changes, theme, cx),
            Err(error) => self
                .files_column(cx)
                .child(problem(error.to_string(), theme)),
        }
    }

    fn render_working_lists(
        &self,
        changes: &[Change],
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let states: Vec<FileState> = changes.iter().map(FileState::of).collect();
        self.files_column(cx)
            .child(self.render_file_list(&states, Staging::Staged, theme, cx))
            .child(self.render_file_list(&states, Staging::Unstaged, theme, cx))
    }

    /// The Staged files list or the Unstaged files list.
    fn render_file_list(
        &self,
        states: &[FileState],
        staging: Staging,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let count = states
            .iter()
            .filter(|state| state.listed_in(staging))
            .count();
        let header = self.list_header(staging, theme);
        let body = self.render_file_rows(count, staging, theme, cx);
        list_frame(staging, theme, cx).child(header).child(body)
    }

    fn list_header(&self, staging: Staging, theme: &Theme) -> Div {
        let picked = self.picked_count(staging);
        let count = div()
            .text_size(px(11.))
            .text_color(theme.muted)
            .child(format!("{picked} selected"));
        div()
            .flex()
            .items_center()
            .justify_between()
            .pr_1()
            .bg(theme.surface)
            .child(section_title(list_title(staging), theme))
            .when(picked > 0, |header| header.child(count))
    }

    fn picked_count(&self, staging: Staging) -> usize {
        if self.shown.marked.staging == staging {
            self.shown.marked.paths.len()
        } else {
            0
        }
    }

    /// Each list takes half of the column and scrolls on its own. Only the
    /// rows in view are built, so a long list costs the same as a short one.
    fn render_file_rows(
        &self,
        count: usize,
        staging: Staging,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> UniformList {
        let theme = *theme;
        let rows = cx.processor(move |this, range: Range<usize>, _window, cx| {
            this.visible_rows(range, staging, &theme, cx)
        });
        let id = SharedString::from(format!("rows-{}", side_name(staging)));
        uniform_list(id, count, rows)
            .track_scroll(self.list_scroll(staging).clone())
            .flex_1()
            .min_h(px(0.))
    }

    pub(super) fn visible_rows(
        &self,
        range: Range<usize>,
        staging: Staging,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let Some(Ok(changes)) = self.shown.detail.as_ref().map(|detail| &detail.changes) else {
            return Vec::new();
        };
        let listed = changes.iter().map(|change| (change, FileState::of(change)));
        let visible = listed
            .filter(|(_, state)| state.listed_in(staging))
            .skip(range.start)
            .take(range.len());
        visible
            .map(|(change, state)| {
                let row = FileRow::new(change, state, staging);
                self.render_file_row(&row, theme, cx).into_any_element()
            })
            .collect()
    }

    /// A row is highlighted while its file is picked or its diff is open.
    fn render_file_row(
        &self,
        row: &FileRow,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (path, staging) = (row.file.path.as_str(), row.file.staging);
        let background = self.is_highlighted(path, staging).then_some(theme.selected);
        let dragged = self.dragged_with(path, staging);
        file_row_frame(background).child(file_row_button(row, dragged, theme, cx))
    }

    fn is_highlighted(&self, path: &str, staging: Staging) -> bool {
        let viewing = self.shown.viewing.as_ref();
        let open = viewing.is_some_and(|viewing| viewing.shows(path, staging));
        self.shown.marked.holds(path, staging) || open
    }

    /// Dragging one of several picked files takes all of them along.
    fn dragged_with(&self, path: &str, staging: Staging) -> Vec<String> {
        let marked = &self.shown.marked;
        if !marked.holds(path, staging) || marked.paths.len() <= 1 {
            return vec![path.to_string()];
        }
        let listed = self.list_files(staging).into_iter().map(|(file, _)| file);
        listed.filter(|file| marked.paths.contains(file)).collect()
    }
}

fn list_title(staging: Staging) -> &'static str {
    match staging {
        Staging::Staged => "Staged files",
        Staging::Unstaged => "Unstaged files",
    }
}
