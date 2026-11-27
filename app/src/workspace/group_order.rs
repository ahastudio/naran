//! Dragging a group row onto another group row to reorder the groups.

use gpui::{Context, Div, Stateful, prelude::*};

use super::Workspace;
use super::repositories::DraggedRepository;
use crate::theme::Theme;
use crate::widgets::{drag_preview, drop_border};

/// A group row being dragged to another place in the group column.
#[derive(Clone)]
struct DraggedGroup {
    name: String,
}

impl Workspace {
    /// Puts a dragged group into the slot of the row it was dropped on, and
    /// keeps that order. The selection does not change.
    pub(super) fn move_group(&mut self, from: &str, to: &str, cx: &mut Context<Self>) {
        if self.settings.move_group(from, to) {
            self.save_settings();
            cx.notify();
        }
    }
}

/// Makes the row of the group `name` something to drag, and a place where
/// another group row can be dropped.
pub(super) fn reorderable(
    row: Stateful<Div>,
    name: &str,
    theme: &Theme,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let (tint, target) = (theme.accent, name.to_owned());
    let dragged = DraggedGroup {
        name: name.to_owned(),
    };
    let row = row
        .on_drag(dragged, |dragged, _, _, cx| drag_preview(&dragged.name, cx))
        .drag_over::<DraggedGroup>(move |style, _, _, _| drop_border(style, tint))
        .on_drop(cx.listener(move |this, dragged: &DraggedGroup, _, cx| {
            this.move_group(&dragged.name, &target, cx);
        }));
    accepting_repositories(row, name, theme, cx)
}

/// Makes the row of the group `name` take a dragged repository row, which
/// moves it into the group.
fn accepting_repositories(
    row: Stateful<Div>,
    name: &str,
    theme: &Theme,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    let (tint, target) = (theme.accent, name.to_owned());
    let marked = target.clone();
    row.drag_over::<DraggedRepository>(move |style, dragged, _, _| {
        if dragged.moves_onto(&marked) {
            drop_border(style, tint)
        } else {
            style
        }
    })
    .on_drop(
        cx.listener(move |this, dragged: &DraggedRepository, _, cx| {
            this.move_to_group(&dragged.path, &dragged.group, &target, cx);
        }),
    )
}
