//! The group column: picking a group, and creating, renaming and deleting
//! one.

use gpui::{
    AnyElement, Context, Div, Entity, Focusable, MouseButton, PromptLevel, Stateful, Window, div,
    prelude::*, px,
};

use super::Workspace;
use super::group_order::reorderable;
use super::menu::{MenuTarget, open_menu_listener};
use super::state::{EditKind, Editing, Outcome, Pane};
use crate::model;
use crate::text_input::{InputEvent, InputLook, TextInput};
use crate::theme::Theme;
use crate::widgets::{GroupRow, group_row, link, section_title};

impl Workspace {
    pub(super) fn start_editing(
        &mut self,
        kind: EditKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = cx.new(|cx| name_input(&kind.initial_text(), cx));
        let subscription = cx.subscribe_in(&input, window, Self::handle_input);
        window.focus(&input.focus_handle(cx));
        self.interaction.editing = Some(Editing {
            kind,
            input,
            _subscription: subscription,
        });
        cx.notify();
    }

    fn handle_input(
        &mut self,
        _input: &Entity<TextInput>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Submit(text) => self.finish_editing(text, window, cx),
            InputEvent::Cancel => self.stop_editing(window, cx),
        }
    }

    fn stop_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.interaction.editing = None;
        window.focus(&self.focus_handle);
        cx.notify();
    }

    pub(super) fn finish_editing(
        &mut self,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(editing) = self.interaction.editing.take() {
            self.apply_edit(editing.kind, text.trim(), cx);
            window.focus(&self.focus_handle);
            cx.notify();
        }
    }

    fn apply_edit(&mut self, kind: EditKind, name: &str, cx: &mut Context<Self>) {
        match (self.edit_applied(kind, name), name.is_empty()) {
            (true, _) => self.save_and_select(name, cx),
            (false, false) => self.report(
                format!("A group named \"{name}\" already exists."),
                Outcome::Failed,
            ),
            (false, true) => {}
        }
    }

    /// False when the name is empty or another group has it.
    fn edit_applied(&mut self, kind: EditKind, name: &str) -> bool {
        match kind {
            EditKind::NewGroup => self.settings.create_group(name),
            EditKind::RenameGroup(old) => self.settings.rename_group(&old, name),
        }
    }

    fn save_and_select(&mut self, name: &str, cx: &mut Context<Self>) {
        self.save_settings();
        self.select_group(Some(name.to_string()), cx);
    }

    /// Asks first. Nothing is deleted unless the answer is Delete.
    pub(super) fn delete_group(name: String, window: &mut Window, cx: &mut Context<Self>) {
        let answer = window.prompt(
            PromptLevel::Warning,
            &format!("Delete the group \"{name}\"?"),
            Some("Repositories that are only in this group leave Naran. Their folders are not touched."),
            &["Delete", "Cancel"],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await == Ok(0) {
                this.update(cx, |this, cx| this.drop_group(&name, cx)).ok();
            }
        })
        .detach();
    }

    fn drop_group(&mut self, name: &str, cx: &mut Context<Self>) {
        self.settings.delete_group(name);
        self.save_settings();
        self.prune();
        // Another group stays selected when it is not the one deleted.
        let keep = self
            .selection
            .group
            .clone()
            .filter(|selected| selected != name);
        self.select_group(keep, cx);
    }

    pub(super) fn render_groups(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let groups = self.settings.groups.iter().enumerate();
        let items: Vec<AnyElement> = groups
            .filter_map(|(index, group)| self.group_item(index, group, theme, cx))
            .collect();
        let new_group = self.new_group_items(theme, cx);
        self.groups_frame(theme, cx)
            .children(items)
            .children(new_group)
    }

    fn groups_frame(&self, theme: &Theme, cx: &mut Context<Self>) -> Stateful<Div> {
        div()
            .id("groups")
            .debug_selector(|| "groups".into())
            .track_scroll(&self.scrolls.groups)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.focus_pane(Pane::Groups)),
            )
            .w(px(self.layout.sizes.groups))
            .flex_none()
            .h_full()
            .overflow_y_scroll()
            .bg(theme.sidebar)
            .p_2()
            .flex()
            .flex_col()
            .gap_0p5()
            .child(section_title("GROUPS", theme))
    }

    /// The row of a group, or the input when it is being renamed.
    fn group_item(
        &self,
        index: usize,
        group: &model::Group,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if self.is_renaming(group) {
            return self.render_input(theme).map(IntoElement::into_any_element);
        }
        Some(self.group_entry(index, group, theme, cx))
    }

    fn edit_kind(&self) -> Option<&EditKind> {
        self.interaction
            .editing
            .as_ref()
            .map(|editing| &editing.kind)
    }

    fn is_renaming(&self, group: &model::Group) -> bool {
        matches!(self.edit_kind(), Some(EditKind::RenameGroup(name)) if *name == group.name)
    }

    fn is_creating_group(&self) -> bool {
        matches!(self.edit_kind(), Some(EditKind::NewGroup))
    }

    fn group_entry(
        &self,
        index: usize,
        group: &model::Group,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let members = model::members(&self.entries, group);
        let row = GroupRow {
            name: group.name.clone().into(),
            members: &members,
            selected: self.selection.group.as_deref() == Some(group.name.as_str()),
        };
        let entry = group_row(("group", index), &row, theme);
        let events = group_events(entry, &group.name, cx);
        reorderable(events, &group.name, theme, cx).into_any_element()
    }

    /// The input of a group being created, or the link that starts it.
    fn new_group_items(&self, theme: &Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        if self.is_creating_group() {
            return self
                .render_input(theme)
                .into_iter()
                .map(IntoElement::into_any_element)
                .collect();
        }
        vec![new_group_link(theme, cx)]
    }

    fn render_input(&self, theme: &Theme) -> Option<impl IntoElement> {
        self.interaction.editing.as_ref().map(|editing| {
            div()
                .flex()
                .items_center()
                .h(px(28.))
                .px_2()
                .rounded_md()
                .border_1()
                .border_color(theme.accent)
                .bg(theme.background)
                .child(editing.input.clone())
        })
    }
}

/// What a click and a right-click on the row of the group `name` do.
fn group_events(row: Stateful<Div>, name: &str, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let selected = name.to_owned();
    row.on_click(cx.listener(move |this, _, _, cx| this.select_group(Some(selected.clone()), cx)))
        .on_mouse_down(
            MouseButton::Right,
            open_menu_listener(MenuTarget::Group(name.to_owned()), cx),
        )
}

fn name_input(initial: &str, cx: &mut Context<TextInput>) -> TextInput {
    let theme = Theme::dark();
    TextInput::new(
        initial,
        InputLook {
            placeholder: "Group name".into(),
            placeholder_color: theme.muted,
            cursor_color: theme.accent,
        },
        cx,
    )
}

fn new_group_link(theme: &Theme, cx: &mut Context<Workspace>) -> AnyElement {
    link("new-group", "+ New Group", theme)
        .mt_1()
        .on_click(
            cx.listener(|this, _, window, cx| this.start_editing(EditKind::NewGroup, window, cx)),
        )
        .into_any_element()
}
