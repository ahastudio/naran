//! The sizes of the parts of the window, the dividers that change them, and
//! the layout saved across runs.

use gpui::{
    Context, Div, Hsla, MouseButton, Pixels, Point, SharedString, Stateful, div, prelude::*, px,
};
use serde::{Deserialize, Serialize};

use super::Workspace;
use super::state::{Outcome, Pane, Pressed};
use crate::store;
use crate::theme::Theme;
use crate::widgets::identified;

pub(super) const TOOLBAR_HEIGHT: f32 = 44.;
pub(super) const DIVIDER_WIDTH: f32 = 5.;

/// A boundary between two parts of the window that can be dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Divider {
    Groups,
    Repositories,
    Commits,
    Files,
}

impl Divider {
    /// A divider between two columns is dragged sideways; the one below the
    /// commit list is dragged up and down.
    fn is_vertical(self) -> bool {
        self != Self::Commits
    }
}

/// The smallest and the largest size a part can have, in pixels. A drag and a
/// saved file are held to the same range.
#[derive(Debug, Clone, Copy)]
struct Range {
    low: f32,
    high: f32,
}

impl Range {
    fn clamp(self, value: f32) -> f32 {
        value.clamp(self.low, self.high)
    }
}

const GROUPS: Range = Range {
    low: 120.,
    high: 400.,
};
const REPOSITORIES: Range = Range {
    low: 200.,
    high: 700.,
};
const COMMITS: Range = Range {
    low: 80.,
    high: 700.,
};
const FILES: Range = Range {
    low: 180.,
    high: 800.,
};

/// Sizes of the parts of the window, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Sizes {
    pub(super) groups: f32,
    pub(super) repositories: f32,
    pub(super) commits: f32,
    pub(super) files: f32,
}

impl Sizes {
    pub(super) const DEFAULT: Self = Self {
        groups: 200.,
        repositories: 300.,
        commits: 220.,
        files: 320.,
    };

    /// Moves the sizes the way a divider dragged to `(x, y)` asks for.
    fn drag(&mut self, divider: Divider, (x, y): (f32, f32)) {
        match divider {
            Divider::Groups => self.groups = GROUPS.clamp(x),
            Divider::Repositories => {
                self.repositories = REPOSITORIES.clamp(x - self.groups - DIVIDER_WIDTH);
            }
            Divider::Commits => self.commits = COMMITS.clamp(y - TOOLBAR_HEIGHT),
            Divider::Files => {
                let left = self.groups + self.repositories + 2. * DIVIDER_WIDTH;
                self.files = FILES.clamp(x - left);
            }
        }
    }

    /// Keeps every size inside the range a drag can reach, so a hand-edited
    /// or broken file cannot make a part vanish.
    pub(super) fn clamped(self) -> Self {
        let fit = |value: f32, range: Range, default: f32| {
            if value.is_finite() {
                range.clamp(value)
            } else {
                default
            }
        };
        Self {
            groups: fit(self.groups, GROUPS, Self::DEFAULT.groups),
            repositories: fit(self.repositories, REPOSITORIES, Self::DEFAULT.repositories),
            commits: fit(self.commits, COMMITS, Self::DEFAULT.commits),
            files: fit(self.files, FILES, Self::DEFAULT.files),
        }
    }
}

impl Default for Sizes {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The part of the screen state that the user shapes: part sizes and the list
/// the arrow keys move through. Saved in `layout.json`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct SavedLayout {
    pub(super) sizes: Sizes,
    pub(super) pane: Pane,
    /// The commit graph also draws the refs Git keeps for itself.
    pub(super) refs: naran_core::Refs,
}

impl SavedLayout {
    /// The saved layout, with every part at least its smallest size.
    pub(super) fn clamped(self) -> Self {
        Self {
            sizes: self.sizes.clamped(),
            ..self
        }
    }
}

impl Workspace {
    /// Follows the pointer while a divider is held.
    pub(super) fn drag(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        if let Some(divider) = self.interaction.dragging {
            self.layout
                .sizes
                .drag(divider, (f32::from(position.x), f32::from(position.y)));
            self.save_layout();
            cx.notify();
        }
    }

    pub(super) fn release_drag(&mut self, cx: &mut Context<Self>) {
        if self.interaction.dragging.take().is_some() {
            cx.notify();
        }
    }

    /// A thin handle between two parts of the window. Hold it and move the
    /// pointer to resize.
    pub(super) fn divider(
        &self,
        which: Divider,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let hover = theme.accent.opacity(0.35);
        divider_handle(which, theme.border)
            .when(self.interaction.dragging == Some(which), |style| {
                style.bg(hover)
            })
            .hover(move |style| style.bg(hover))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| this.start_drag(which, cx)),
            )
    }

    pub(super) fn start_drag(&mut self, which: Divider, cx: &mut Context<Self>) {
        self.interaction.dragging = Some(which);
        cx.notify();
    }

    pub(super) fn save_layout(&mut self) {
        let layout = SavedLayout {
            sizes: self.layout.sizes,
            pane: self.layout.pane,
            refs: self.layout.refs,
        };
        if let Err(error) = store::save(&self.files.layout, &layout) {
            self.report(
                format!("Could not save the layout: {error}"),
                Outcome::Failed,
            );
        }
    }

    /// The list the arrow keys move through. Remembered across runs.
    pub(super) fn focus_pane(&mut self, pane: Pane) {
        self.interaction.pressed = Pressed::List;
        if self.layout.pane != pane {
            self.layout.pane = pane;
            self.save_layout();
        }
    }
}

fn divider_handle(which: Divider, line: Hsla) -> Stateful<Div> {
    let base = identified(SharedString::from(format!("divider-{which:?}")))
        .flex_none()
        .flex()
        .justify_center();
    if which.is_vertical() {
        return base
            .w(px(DIVIDER_WIDTH))
            .h_full()
            .cursor_col_resize()
            .child(div().w(px(1.)).h_full().bg(line));
    }
    base.h(px(DIVIDER_WIDTH))
        .w_full()
        .items_center()
        .cursor_row_resize()
        .child(div().h(px(1.)).w_full().bg(line))
}
