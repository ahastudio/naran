//! The picked commit on screen: its files above its message on the left,
//! and the diff of the open file on the right. Nothing here can be staged.

use std::cell::RefCell;
use std::ops::Range;

use gpui::{
    AnyElement, Context, Div, FontWeight, Hsla, MouseButton, SharedString, Stateful, UniformList,
    div, prelude::*, px, uniform_list,
};
use naran_core::CommitFile;

use super::super::Workspace;
use super::super::state::Pressed;
use super::commit::Inspecting;
use super::commit_info::render_commit_info;
use super::diff_rows::{DIFF_LIMIT, DiffLook, named_lines, number_width, omitted_note, plain_rows};
use super::diff_view::lines_content;
use super::pictures::{image_format, render_pictures};
use crate::model;
use crate::selectable::{Texts, Which, area};
use crate::theme::Theme;
use crate::widgets::{muted, problem, section_title, status_badge};

impl Workspace {
    /// Files above, the description below, each half scrolling on its own.
    pub(super) fn render_commit_side(
        &self,
        inspecting: &Inspecting,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let files = div()
            .id("commit-files")
            .debug_selector(|| "commit-files-list".into())
            .flex_1()
            .min_h(px(0.))
            .border_b_1()
            .border_color(theme.border)
            .child(self.render_commit_files(inspecting, theme, cx));
        let info = div()
            .id("commit-info")
            .debug_selector(|| "commit-info".into())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.interaction.pressed = Pressed::Text),
            )
            .flex_1()
            .min_h(px(0.))
            .overflow_y_scroll()
            .child(render_commit_info(inspecting, theme, &self.texts));
        self.files_column(cx).child(files).child(info)
    }

    fn render_commit_files(
        &self,
        inspecting: &Inspecting,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let column = div().size_full().flex().flex_col();
        match commit_files_content(inspecting, theme) {
            Ok(files) => self.render_commit_file_list(column, files.len(), theme, cx),
            Err(shown) => column.child(shown).into_any_element(),
        }
    }

    fn render_commit_file_list(
        &self,
        column: Div,
        count: usize,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let title = div()
            .flex_none()
            .bg(theme.surface)
            .child(section_title(format!("Files ({count})"), theme));
        let list = self.commit_file_list(count, *theme, cx);
        column.child(title).child(list).into_any_element()
    }

    fn commit_file_list(&self, count: usize, theme: Theme, cx: &mut Context<Self>) -> UniformList {
        let rows = cx.processor(move |this, range: Range<usize>, _window, cx| {
            this.commit_file_rows(range, &theme, cx)
        });
        uniform_list("commit-file-rows", count, rows)
            .track_scroll(self.scrolls.commit_files.clone())
            .flex_1()
            .min_h(px(0.))
    }

    /// The rows `range` of the file list of the picked commit.
    pub(super) fn commit_file_rows(
        &mut self,
        range: Range<usize>,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let inspecting = self.shown.inspecting.as_ref();
        let Some((inspecting, Some(Ok(files)))) =
            inspecting.map(|inspecting| (inspecting, inspecting.files.as_ref()))
        else {
            return Vec::new();
        };
        let background = |file: &CommitFile| {
            let picked = inspecting.is_open(&file.path) || inspecting.holds(&file.path);
            picked.then_some(theme.selected)
        };
        range
            .filter_map(|index| files.get(index))
            .map(|file| commit_file_row(file, background(file), theme, cx))
            .collect()
    }

    /// The diff of the open file of the picked commit.
    pub(super) fn render_commit_diff(&self, inspecting: &Inspecting, theme: &Theme) -> AnyElement {
        let Some(file) = inspecting.file.as_ref() else {
            return muted("Select a file to see its changes.", theme);
        };
        match commit_diff_content(inspecting, file, theme) {
            Ok(lines) => named_lines(
                commit_diff_name(file, theme),
                area(
                    commit_diff_text(lines, theme, &self.texts),
                    &self.texts,
                    Which::Diff,
                ),
                &self.scrolls.diff_sideways,
            )
            .into_any_element(),
            Err(shown) => shown,
        }
    }
}

/// The rows of the Files list of a commit, or the element that says why there are none.
fn commit_files_content<'a>(
    inspecting: &'a Inspecting,
    theme: &Theme,
) -> Result<&'a [CommitFile], AnyElement> {
    match &inspecting.files {
        None => Err(muted("Reading…", theme)),
        Some(Err(error)) => Err(problem(error.to_string(), theme)),
        Some(Ok(files)) if files.is_empty() => {
            Err(muted("No files changed in this commit.", theme))
        }
        Some(Ok(files)) => Ok(files),
    }
}

/// A file of the commit, on `background` when it is the open one.
fn commit_file_row(
    file: &CommitFile,
    background: Option<Hsla>,
    theme: &Theme,
    cx: &mut Context<Workspace>,
) -> AnyElement {
    let (path, hover) = (file.path.clone(), theme.hover);
    let target = path.clone();
    div()
        .id(SharedString::from(format!("commit-file-{path}")))
        .debug_selector(|| format!("commit-file-{path}"))
        .w_full()
        .h(px(26.))
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .cursor_pointer()
        .when_some(background, Styled::bg)
        .hover(move |style| style.bg(hover))
        .on_click(cx.listener(move |this, event: &gpui::ClickEvent, _, cx| {
            this.click_commit_file(target.clone(), event.modifiers(), cx);
        }))
        .child(status_badge(badge_letter(file.status), theme))
        .child(div().flex_1().min_w(px(0.)).truncate().child(path))
        .into_any_element()
}

pub(super) fn badge_letter(status: char) -> char {
    match status {
        letter @ ('A' | 'D' | 'R' | 'C' | 'T') => letter,
        _ => 'M',
    }
}

/// The lines of the open file of the commit, or the element that says why there are none.
fn commit_diff_content<'a>(
    inspecting: &'a Inspecting,
    file: &str,
    theme: &Theme,
) -> Result<&'a [String], AnyElement> {
    if image_format(file).is_some() {
        return Err(render_pictures(file, inspecting.pictures.as_ref(), theme));
    }
    lines_content(inspecting.lines.as_ref(), theme)
}

/// The rows of the diff of a commit's file; their selectable text is handed to `texts`.
fn commit_diff_text(lines: &[String], theme: &Theme, texts: &Texts) -> Div {
    let rows = model::diff_rows(&lines[..lines.len().min(DIFF_LIMIT)]);
    let look = DiffLook {
        id: "commit-diff-line",
        theme,
        region: &texts.diff,
        number_width: number_width(&rows),
        texts: RefCell::default(),
    };
    let body = plain_rows(&rows, &look);
    texts.diff.set_lines(look.take_texts());
    omitted_note(body, lines.len(), theme)
}

fn commit_diff_name(file: &str, theme: &Theme) -> Div {
    div()
        .debug_selector(|| "diff-name".into())
        .flex_none()
        .px_2()
        .py_1()
        .bg(theme.surface)
        .truncate()
        .font_weight(FontWeight::SEMIBOLD)
        .child(file.to_string())
}
