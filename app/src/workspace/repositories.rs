//! The repository column: the selected group's repositories, the notices
//! above them, and dragging a row to another place in the group.

use std::path::{Path, PathBuf};

use gpui::{Context, Div, FontWeight, MouseButton, Stateful, div, prelude::*, px};

use super::Workspace;
use super::menu::{MenuTarget, open_menu_listener};
use super::state::Pane;
use crate::model::Entry;
use crate::theme::Theme;
use crate::widgets::{drag_preview, link, repository_row};

/// A repository row being dragged to another place in its group.
#[derive(Clone)]
struct DraggedRepository {
    path: PathBuf,
    name: String,
}

impl Workspace {
    pub(super) fn render_repositories(
        &self,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let members = self.members();
        let header = self.repositories_header(members.len(), theme, cx);
        let list = self.repositories_list(&members, theme, cx);
        div()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.focus_pane(Pane::Repositories)),
            )
            .w(px(self.layout.sizes.repositories))
            .flex_none()
            .h_full()
            .flex()
            .flex_col()
            .child(header)
            .child(list)
    }

    fn repositories_header(&self, count: usize, theme: &Theme, cx: &mut Context<Self>) -> Div {
        let group = self.group();
        let name = group.map_or("No group", |group| group.name.as_str());
        column_header(theme)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .truncate()
                    .child(name.to_owned()),
            )
            .child(
                div()
                    .pl_1()
                    .text_color(theme.muted)
                    .child(count.to_string()),
            )
            .child(div().flex_1())
            .children(group.map(|_| add_repository_link(theme, cx)))
    }

    fn repositories_list(
        &self,
        members: &[&Entry],
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let rows: Vec<Div> = members
            .iter()
            .enumerate()
            .map(|(index, entry)| self.repository_item(index, entry, theme, cx))
            .collect();
        self.repositories_frame()
            .children(
                self.progress
                    .picking_folders
                    .then(|| picking_banner(theme, cx)),
            )
            .children(
                self.feedback
                    .notice
                    .clone()
                    .map(|notice| notice_card(notice, theme, cx)),
            )
            .children(members.is_empty().then(|| self.empty_note(theme)))
            .children(rows)
    }

    fn empty_note(&self, theme: &Theme) -> Div {
        div().p_3().text_color(theme.muted).child(self.empty_text())
    }

    fn empty_text(&self) -> &'static str {
        if self.progress.reading {
            return "Reading…";
        }
        if self.group().is_some() {
            return "This group is empty. Use + Add Repository, or drop folders onto the window.";
        }
        "Create a group with + New Group first. Repositories live inside groups."
    }

    fn repository_item(
        &self,
        index: usize,
        entry: &Entry,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Div {
        let path = entry.path().to_path_buf();
        let selected = self.selection.repository.as_deref() == Some(entry.path());
        let content = repository_content(index, entry, theme, cx);
        repository_frame(&path, theme, cx)
            .when(selected, |row| row.bg(theme.selected))
            .child(content)
            .on_mouse_down(
                MouseButton::Right,
                open_menu_listener(MenuTarget::Repository(path), cx),
            )
    }

    fn repositories_frame(&self) -> Stateful<Div> {
        div()
            .id("repositories")
            .track_scroll(&self.scrolls.repositories)
            .flex_1()
            .overflow_y_scroll()
            .flex()
            .flex_col()
    }

    pub(super) fn dismiss_notice(&mut self, cx: &mut Context<Self>) {
        self.feedback.notice = None;
        cx.notify();
    }
}

/// The header above a column: its title, a count and its link.
fn column_header(theme: &Theme) -> Div {
    div()
        .flex_none()
        .flex()
        .items_center()
        .gap_1()
        .px_3()
        .h(px(40.))
        .border_b_1()
        .border_color(theme.border)
}

fn add_repository_link(theme: &Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    link("add-repository", "+ Add Repository", theme)
        .on_click(cx.listener(|this, _, _, cx| this.prompt_for_repositories(cx)))
}

fn picking_banner(theme: &Theme, cx: &mut Context<Workspace>) -> Div {
    let text = "Waiting for the system folder picker… It can take a few seconds to open.";
    let cancel = link("cancel-picking", "Cancel", theme)
        .on_click(cx.listener(|this, _, _, _| this.services.picker.cancel()));
    div()
        .m_2()
        .p_2()
        .rounded_md()
        .border_1()
        .border_color(theme.accent.opacity(0.5))
        .bg(theme.accent.opacity(0.10))
        .text_color(theme.text)
        .flex()
        .items_center()
        .gap_2()
        .child(div().flex_1().min_w(px(0.)).child(text))
        .child(cancel)
}

fn notice_card(notice: String, theme: &Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let hint = div()
        .mt_1()
        .text_size(px(11.))
        .text_color(theme.muted)
        .child("Click to dismiss");
    div()
        .id("notice")
        .debug_selector(|| "notice".into())
        .m_2()
        .p_2()
        .rounded_md()
        .border_1()
        .border_color(theme.problem.opacity(0.5))
        .bg(theme.problem.opacity(0.12))
        .text_color(theme.problem)
        .cursor_pointer()
        .child(notice)
        .child(hint)
        .on_click(cx.listener(|this, _, _, cx| this.dismiss_notice(cx)))
}

fn repository_frame(path: &Path, theme: &Theme, cx: &mut Context<Workspace>) -> Div {
    let (tint, drop_path) = (theme.accent, path.to_path_buf());
    div()
        .group("repository-row")
        .drag_over::<DraggedRepository>(move |style, _, _, _| style.border_t_2().border_color(tint))
        .on_drop(
            cx.listener(move |this, dragged: &DraggedRepository, _, cx| {
                this.move_repository(&dragged.path, &drop_path, cx);
            }),
        )
        .border_b_1()
        .border_color(theme.border)
        .flex()
        .items_center()
}

/// The row itself: clicked to select the repository, dragged to move it.
fn repository_content(
    index: usize,
    entry: &Entry,
    theme: &Theme,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let select_path = entry.path().to_path_buf();
    let dragged = DraggedRepository {
        path: entry.path().to_path_buf(),
        name: entry.name(),
    };
    repository_row(index, entry, theme)
        .flex_1()
        .min_w(px(0.))
        .on_drag(dragged, |dragged, _, _, cx| drag_preview(&dragged.name, cx))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.select_repository(Some(select_path.clone()), cx);
        }))
}
