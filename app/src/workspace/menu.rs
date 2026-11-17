//! The small menu opened by pressing the right button on a group or a
//! repository.

use std::path::{Path, PathBuf};

use gpui::{
    AnyElement, App, Context, Div, MouseButton, MouseDownEvent, Pixels, Point, Stateful, Window,
    div, prelude::*, px,
};

use super::Workspace;
use super::state::EditKind;
use crate::theme::Theme;

/// What a right-button press landed on.
#[derive(Debug, Clone)]
pub(super) enum MenuTarget {
    Group(String),
    Repository(PathBuf),
}

/// The small menu opened by pressing the right button on a group or a
/// repository.
#[derive(Debug, Clone)]
pub(super) struct ContextMenu {
    pub(super) target: MenuTarget,
    pub(super) position: Point<Pixels>,
}

/// One choice of the menu: the element id tests find it by, the words shown,
/// and whether it takes something away.
struct MenuChoice {
    id: &'static str,
    label: &'static str,
    tone: Tone,
}

/// A choice that takes something away is drawn in the problem color, so it is
/// not picked by mistake.
#[derive(Clone, Copy)]
enum Tone {
    Plain,
    Destructive,
}

const RENAME: MenuChoice = MenuChoice {
    id: "menu-rename",
    label: "Rename",
    tone: Tone::Plain,
};
const DELETE: MenuChoice = MenuChoice {
    id: "menu-delete",
    label: "Delete…",
    tone: Tone::Destructive,
};
const REVEAL: MenuChoice = MenuChoice {
    id: "menu-reveal",
    label: "Reveal in Finder",
    tone: Tone::Plain,
};
const REMOVE: MenuChoice = MenuChoice {
    id: "menu-remove",
    label: "Remove…",
    tone: Tone::Destructive,
};

impl Workspace {
    fn open_menu(&mut self, target: MenuTarget, position: Point<Pixels>, cx: &mut Context<Self>) {
        self.interaction.menu = Some(ContextMenu { target, position });
        cx.notify();
    }

    /// The right-button menu of a group: a transparent cover that closes it
    /// when pressed, and the two choices on top.
    pub(super) fn render_group_menu(
        &self,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        let menu = self.interaction.menu.clone()?;
        let items = Self::menu_items(&menu.target, theme, cx);
        Some(menu_cover(cx).child(menu_panel(menu.position, theme).children(items)))
    }

    fn menu_items(target: &MenuTarget, theme: &Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        match target {
            MenuTarget::Group(name) => group_menu_items(name, theme, cx),
            MenuTarget::Repository(path) => repository_menu_items(path, theme, cx),
        }
    }

    fn close_menu(&mut self, cx: &mut Context<Self>) {
        self.interaction.menu = None;
        cx.notify();
    }

    fn rename_from_menu(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        self.interaction.menu = None;
        self.start_editing(EditKind::RenameGroup(name), window, cx);
    }

    fn delete_from_menu(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        self.interaction.menu = None;
        Self::delete_group(name, window, cx);
    }

    fn reveal_from_menu(&mut self, path: &Path, cx: &mut Context<Self>) {
        self.interaction.menu = None;
        self.reveal(path);
        cx.notify();
    }

    fn remove_from_menu(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.interaction.menu = None;
        self.confirm_remove(path, window, cx);
    }
}

/// A press of the right button opens the menu of `target` where the pointer is.
pub(super) fn open_menu_listener(
    target: MenuTarget,
    cx: &mut Context<Workspace>,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    cx.listener(move |this, event: &MouseDownEvent, _, cx| {
        this.open_menu(target.clone(), event.position, cx);
    })
}

fn menu_item(choice: &MenuChoice, theme: &Theme) -> Stateful<Div> {
    let hover = theme.hover;
    let color = match choice.tone {
        Tone::Plain => theme.text,
        Tone::Destructive => theme.problem,
    };
    div()
        .id(choice.id)
        .h(px(28.))
        .flex()
        .items_center()
        .px_3()
        .cursor_pointer()
        .text_color(color)
        .hover(move |style| style.bg(hover))
        .child(choice.label)
}

fn group_menu_items(name: &str, theme: &Theme, cx: &mut Context<Workspace>) -> Vec<AnyElement> {
    let (rename, delete) = (name.to_string(), name.to_string());
    let rename_item =
        menu_item(&RENAME, theme).on_click(cx.listener(move |this, _, window, cx| {
            this.rename_from_menu(rename.clone(), window, cx);
        }));
    let delete_item =
        menu_item(&DELETE, theme).on_click(cx.listener(move |this, _, window, cx| {
            this.delete_from_menu(delete.clone(), window, cx);
        }));
    vec![
        rename_item.into_any_element(),
        delete_item.into_any_element(),
    ]
}

fn repository_menu_items(
    path: &Path,
    theme: &Theme,
    cx: &mut Context<Workspace>,
) -> Vec<AnyElement> {
    let (reveal, remove) = (path.to_path_buf(), path.to_path_buf());
    let reveal_item = menu_item(&REVEAL, theme)
        .on_click(cx.listener(move |this, _, _, cx| this.reveal_from_menu(&reveal, cx)));
    let remove_item =
        menu_item(&REMOVE, theme).on_click(cx.listener(move |this, _, window, cx| {
            this.remove_from_menu(remove.clone(), window, cx);
        }));
    vec![
        reveal_item.into_any_element(),
        remove_item.into_any_element(),
    ]
}

/// A transparent cover that closes the menu when pressed.
fn menu_cover(cx: &mut Context<Workspace>) -> Stateful<Div> {
    div()
        .id("group-menu-cover")
        .debug_selector(|| "group-menu-cover".into())
        .absolute()
        .size_full()
        .top_0()
        .left_0()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| this.close_menu(cx)),
        )
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(|this, _, _, cx| this.close_menu(cx)),
        )
}

fn menu_panel(position: Point<Pixels>, theme: &Theme) -> Div {
    div()
        // A press on the menu itself must not reach the cover,
        // which closes the menu before the item is clicked.
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
        .absolute()
        .left(position.x)
        .top(position.y)
        .w(px(150.))
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.surface)
        .shadow_md()
}
