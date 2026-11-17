//! Dragging a group row onto another group row to reorder the groups.

use gpui::{Context, Div, Stateful, prelude::*};

use super::Workspace;
use crate::theme::Theme;
use crate::widgets::drag_preview;

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
    row.on_drag(dragged, |dragged, _, _, cx| drag_preview(&dragged.name, cx))
        .drag_over::<DraggedGroup>(move |style, _, _, _| style.border_t_2().border_color(tint))
        .on_drop(cx.listener(move |this, dragged: &DraggedGroup, _, cx| {
            this.move_group(&dragged.name, &target, cx);
        }))
}
