//! A row of the Staged or Unstaged list, and dragging rows from one list to
//! the other to stage or unstage them.

use gpui::{
    Context, Div, Hsla, Pixels, Render, SharedString, Stateful, Window, div, prelude::*, px,
};
use naran_core::{Change, Staging};

use super::super::Workspace;
use super::marks::ListedFile;
use super::truncated;
use crate::model::{self, FileState};
use crate::theme::Theme;
use crate::widgets::status_badge;

/// The word the list of `staging` is named with in element ids.
pub(super) fn side_name(staging: Staging) -> &'static str {
    match staging {
        Staging::Staged => "staged",
        Staging::Unstaged => "unstaged",
    }
}

/// A file row being dragged between the Staged and Unstaged lists.
#[derive(Clone)]
pub(super) struct DraggedFile {
    /// The picked files when the dragged row is one of them, otherwise just this file.
    pub(super) paths: Vec<String>,
    pub(super) from: Staging,
}

/// What follows the pointer while a file is dragged.
pub(super) struct DragPreview {
    pub(super) name: String,
}

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::dark();
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(theme.accent)
            .bg(theme.surface)
            .text_color(theme.text)
            .child(self.name.clone())
    }
}

pub(super) fn drag_preview(
    dragged: &DraggedFile,
    _offset: gpui::Point<Pixels>,
    _window: &mut Window,
    cx: &mut gpui::App,
) -> gpui::Entity<DragPreview> {
    let name = match dragged.paths.as_slice() {
        [only] => only.clone(),
        several => format!("{} files", several.len()),
    };
    cx.new(|_| DragPreview { name })
}

/// One list, which takes the files dropped on it from the other list.
pub(super) fn list_frame(staging: Staging, theme: &Theme, cx: &mut Context<Workspace>) -> Div {
    let tint = theme.accent.opacity(0.12);
    div()
        .debug_selector(|| format!("list-{}", side_name(staging)))
        .flex_1()
        .min_h(px(0.))
        .flex()
        .flex_col()
        .when(staging == Staging::Staged, |list| {
            list.border_b_1().border_color(theme.border)
        })
        .drag_over::<DraggedFile>(move |style, _, _, _| style.bg(tint))
        .on_drop(cx.listener(move |this, dragged: &DraggedFile, _, cx| {
            this.move_files(dragged.paths.clone(), dragged.from, staging, cx);
        }))
}

/// A file row of the Staged or Unstaged list.
pub(super) struct FileRow {
    pub(super) file: ListedFile,
    letter: char,
    label: String,
}

impl FileRow {
    pub(super) fn new(change: &Change, state: FileState, staging: Staging) -> Self {
        Self {
            file: ListedFile::new(change.path.clone(), staging, state),
            letter: model::change_badge(change, staging),
            label: model::change_label(change, staging),
        }
    }
}

/// The frame of a file row, on `background` when it is highlighted.
pub(super) fn file_row_frame(background: Option<Hsla>) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap_1()
        .px_2()
        .py_0p5()
        .when_some(background, Styled::bg)
}

/// The row itself: a click opens the file, a drag takes `dragged` along.
pub(super) fn file_row_button(
    row: &FileRow,
    dragged: Vec<String>,
    theme: &Theme,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let file = row.file.clone();
    let from = file.staging;
    file_button_frame(&row.file, theme)
        .on_click(cx.listener(move |this, event: &gpui::ClickEvent, _, cx| {
            this.click_file(file.clone(), event.modifiers(), cx);
        }))
        .on_drag(
            DraggedFile {
                paths: dragged,
                from,
            },
            drag_preview,
        )
        .child(status_badge(row.letter, theme))
        .child(truncated(row.label.clone()).debug_selector(|| label_selector(row)))
}

/// The selector of the text of `row`, which holds the whole text.
fn label_selector(row: &FileRow) -> String {
    format!("label-{}-{}", side_name(row.file.staging), row.label)
}

fn file_button_frame(file: &ListedFile, theme: &Theme) -> Stateful<Div> {
    let (side, hover) = (side_name(file.staging), theme.hover);
    div()
        .id(SharedString::from(format!("open-{side}-{}", file.path)))
        .debug_selector(|| format!("file-{side}-{}", file.path))
        .flex_1()
        .min_w(px(0.))
        .flex()
        .items_center()
        .gap_2()
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
}
