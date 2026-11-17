//! The frame of the window: the toolbar at the top, the three columns, the
//! status bar at the bottom, and the keys and drops the whole window takes.

use std::path::Path;

use gpui::{
    Context, Div, ExternalPaths, FontWeight, Hsla, MouseButton, MouseMoveEvent, MouseUpEvent,
    SharedString, Stateful, Window, WindowControlArea, div, prelude::*, px,
};

use super::detail::truncating;
use super::layout::{Divider, TOOLBAR_HEIGHT};
use super::state::Outcome;
use super::{
    AddRepositories, CONTEXT, CopyText, ExtendNext, ExtendPrevious, Refresh, SelectAllText,
    SelectNext, SelectPrevious, Workspace,
};
use crate::model::{self, Entry};
use crate::theme::Theme;
use crate::widgets::{button, disabled_button};

/// The title of the window while no repository is selected.
const APP_NAME: &str = "Naran";

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.show_title(window);
        self.texts.begin_frame();
        let theme = Theme::dark();
        let root = Self::on_pointer(Self::on_keys(self.root_frame(&theme), cx), &theme, cx);
        root.child(self.render_toolbar(&theme, cx))
            .child(self.render_columns(&theme, cx))
            .child(self.render_status_bar(&theme))
            .children(self.render_group_menu(&theme, cx))
    }
}

impl Workspace {
    /// Sets the window title only when it changes, not on every frame.
    fn show_title(&mut self, window: &mut Window) {
        let title = self
            .selection
            .repository
            .as_deref()
            .map_or_else(|| APP_NAME.to_owned(), folder_name);
        if self.title != title {
            window.set_window_title(&title);
            self.title = title;
        }
    }

    fn root_frame(&self, theme: &Theme) -> Div {
        div()
            .size_full()
            .flex()
            .flex_col()
            .relative()
            .key_context(CONTEXT)
            .track_focus(&self.focus_handle)
            .bg(theme.background)
            .text_color(theme.text)
            .text_size(px(13.))
    }

    fn on_keys(root: Div, cx: &mut Context<Self>) -> Div {
        root.on_action(cx.listener(|this, _: &SelectNext, _, cx| this.step(1, cx)))
            .on_action(cx.listener(|this, _: &SelectPrevious, _, cx| this.step(-1, cx)))
            .on_action(cx.listener(|this, _: &ExtendNext, _, cx| this.extend(1, cx)))
            .on_action(cx.listener(|this, _: &ExtendPrevious, _, cx| this.extend(-1, cx)))
            .on_action(cx.listener(|this, _: &Refresh, _, cx| this.refresh(cx)))
            .on_action(
                cx.listener(|this, _: &AddRepositories, _, cx| this.prompt_for_repositories(cx)),
            )
            .on_action(cx.listener(|this, _: &CopyText, _, cx| this.copy_text(cx)))
            .on_action(
                cx.listener(|this, _: &SelectAllText, window, cx| this.select_all(window, cx)),
            )
    }

    fn on_pointer(root: Div, theme: &Theme, cx: &mut Context<Self>) -> Div {
        let accent = theme.accent;
        root.on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
            this.add_repositories(paths.paths().to_vec(), cx);
        }))
        .drag_over::<ExternalPaths>(move |style, _, _, _| style.border_2().border_color(accent))
        .on_mouse_move(
            cx.listener(|this, event: &MouseMoveEvent, _, cx| this.drag(event.position, cx)),
        )
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _: &MouseUpEvent, _, cx| this.release_drag(cx)),
        )
    }

    pub(super) fn copy_text(&mut self, cx: &mut Context<Self>) {
        if let Some(text) = self.texts.copied() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        }
    }

    fn render_columns(&self, theme: &Theme, cx: &mut Context<Self>) -> Div {
        div()
            .flex_1()
            .flex()
            .flex_row()
            .overflow_hidden()
            .child(self.render_groups(theme, cx))
            .child(self.divider(Divider::Groups, theme, cx))
            .child(self.render_repositories(theme, cx))
            .child(self.divider(Divider::Repositories, theme, cx))
            .child(self.render_detail(theme, cx))
    }

    fn render_toolbar(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        // A button that is faded must not answer clicks.
        let reading = self.progress.reading;
        let refresh = self.refresh_button(theme).when(!reading, |button| {
            button.on_click(cx.listener(|this, _, _, cx| this.refresh(cx)))
        });
        toolbar_frame(theme)
            .child(title_label(self.title.clone()))
            .child(div().flex_1())
            .child(refresh)
    }

    /// Refresh waits while the repositories are being read.
    fn refresh_button(&self, theme: &Theme) -> Stateful<Div> {
        if self.progress.reading {
            disabled_button("refresh", "Reading…", theme)
        } else {
            button("refresh", "Refresh", theme)
        }
    }

    fn render_status_bar(&self, theme: &Theme) -> impl IntoElement {
        let (text, outcome) = self.status_text(&self.members());
        let message = truncating()
            .text_color(message_color(outcome, theme))
            .child(text);
        status_bar_frame(theme).child(message).children(
            self.selection
                .repository
                .as_ref()
                .map(|path| path_label(path, theme)),
        )
    }

    /// The last message, or how the group stands.
    fn status_text(&self, members: &[&Entry]) -> (SharedString, Outcome) {
        self.feedback
            .message
            .clone()
            .unwrap_or_else(|| (self.summary(members).into(), Outcome::Succeeded))
    }

    fn summary(&self, members: &[&Entry]) -> String {
        match self.group() {
            Some(group) => format!(
                "{}: {} repositories, {} need attention",
                group.name,
                members.len(),
                members.iter().filter(|entry| entry.has_problem()).count()
            ),
            None => "No groups yet.".to_string(),
        }
    }
}

pub(super) fn folder_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The bar at the top, which also moves the window when it is dragged. The
/// traffic lights sit on its left.
fn toolbar_frame(theme: &Theme) -> Div {
    div()
        .h(px(TOOLBAR_HEIGHT))
        .flex_none()
        .flex()
        .items_center()
        .gap_2()
        .pl(px(80.))
        .pr_3()
        .window_control_area(WindowControlArea::Drag)
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.sidebar)
        .relative()
}

/// The title in the middle of the toolbar, whatever is on the left (the
/// window buttons) and on the right (Refresh). macOS hides the system title
/// of a window whose title bar is see-through, so it is drawn here.
fn title_label(title: String) -> Div {
    div()
        .debug_selector(|| "window-title".into())
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .font_weight(FontWeight::BOLD)
        .text_size(px(15.))
        .child(title)
}

fn message_color(outcome: Outcome, theme: &Theme) -> Hsla {
    match outcome {
        Outcome::Succeeded => theme.muted,
        Outcome::Failed => theme.problem,
    }
}

fn status_bar_frame(theme: &Theme) -> Div {
    div()
        .h(px(26.))
        .flex_none()
        .flex()
        .items_center()
        .px_3()
        .border_t_1()
        .border_color(theme.border)
        .bg(theme.sidebar)
        .text_size(px(12.))
        .gap_3()
}

fn path_label(path: &Path, theme: &Theme) -> Div {
    // Where the selected repository lives, always in view.
    div()
        .flex_none()
        .max_w(px(520.))
        .truncate()
        .text_color(theme.muted)
        .child(model::display_path(path, dirs::home_dir().as_deref(), 70))
}
