//! Naran: a Git client that shows many repositories side by side. The
//! executable in `main.rs` only starts the platform's event loop with [`start`].
#![cfg_attr(
    test,
    allow(clippy::too_many_lines, reason = "a test reads from top to bottom")
)]

mod finder;
mod model;
mod picker;
mod selectable;
mod store;
mod text_input;
mod theme;
mod watcher;
mod widgets;
mod workspace;

use std::path::Path;

use gpui::{
    App, AppContext, Bounds, Entity, Focusable, Pixels, TitlebarOptions, Window, WindowBounds,
    WindowOptions, point, px, size,
};

use serde::{Deserialize, Serialize};

pub use crate::store::config_directory;
use crate::workspace::Workspace;

/// Which state the window was in.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode {
    #[default]
    Windowed,
    Maximized,
    Fullscreen,
}

/// Where the window was last, saved in the app data folder. The size is
/// the one the window has when it is not maximized or full screen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct SavedWindow {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    #[serde(default)]
    mode: Mode,
}

impl SavedWindow {
    fn of(window_bounds: WindowBounds) -> Self {
        let (mode, bounds) = match window_bounds {
            WindowBounds::Windowed(bounds) => (Mode::Windowed, bounds),
            WindowBounds::Maximized(bounds) => (Mode::Maximized, bounds),
            WindowBounds::Fullscreen(bounds) => (Mode::Fullscreen, bounds),
        };
        Self {
            x: f32::from(bounds.origin.x),
            y: f32::from(bounds.origin.y),
            width: f32::from(bounds.size.width),
            height: f32::from(bounds.size.height),
            mode,
        }
    }

    /// The saved place, moved onto a screen when it is on none, in the saved mode.
    fn window_bounds(&self, screens: &[Bounds<Pixels>]) -> WindowBounds {
        let bounds = keep_on_screen(self.bounds(), screens);
        match self.mode {
            Mode::Windowed => WindowBounds::Windowed(bounds),
            Mode::Maximized => WindowBounds::Maximized(bounds),
            Mode::Fullscreen => WindowBounds::Fullscreen(bounds),
        }
    }

    fn bounds(&self) -> Bounds<Pixels> {
        Bounds::new(
            point(px(self.x), px(self.y)),
            size(px(self.width), px(self.height)),
        )
    }
}

/// Keeps the size exactly. When the saved place is on no screen any more
/// (a display was unplugged), only the position moves, onto the first screen.
fn keep_on_screen(bounds: Bounds<Pixels>, screens: &[Bounds<Pixels>]) -> Bounds<Pixels> {
    if screens.is_empty() || screens.iter().any(|screen| screen.intersects(&bounds)) {
        return bounds;
    }
    move_onto(bounds, screens[0])
}

/// The position nearest to `bounds` that keeps all of it on `screen`, as far as it fits.
fn move_onto(bounds: Bounds<Pixels>, screen: Bounds<Pixels>) -> Bounds<Pixels> {
    let right = (screen.right() - bounds.size.width).max(screen.origin.x);
    let bottom = (screen.bottom() - bounds.size.height).max(screen.origin.y);
    let x = bounds.origin.x.clamp(screen.origin.x, right);
    let y = bounds.origin.y.clamp(screen.origin.y, bottom);
    Bounds::new(point(x, y), bounds.size)
}

fn save_window(file: &Path, window: &Window) {
    store::save(file, &Some(SavedWindow::of(window.window_bounds()))).ok();
}

const WINDOW_FILE: &str = "window.json";

/// Sets up the keys and opens the main window, with its settings in `directory`.
pub fn start(cx: &mut App, directory: &Path) {
    text_input::bind_keys(cx);
    workspace::bind_keys(cx);
    open_main_window(cx, directory);
    quit_with_last_window(cx);
    cx.activate(true);
}

fn open_main_window(cx: &mut App, directory: &Path) {
    let window_file = directory.join(WINDOW_FILE);
    let options = window_options(initial_window_bounds(&window_file, cx));
    let directory = directory.to_path_buf();
    let opened = cx.open_window(options, move |window, cx| {
        build_workspace(window, cx, &directory, &window_file)
    });
    quit_unless_opened(&opened, cx);
}

/// Without its window the app has nothing to show.
fn quit_unless_opened<T, E>(opened: &Result<T, E>, cx: &mut App) {
    if opened.is_err() {
        cx.quit();
    }
}

fn initial_window_bounds(window_file: &Path, cx: &App) -> WindowBounds {
    let default_bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
    let screens: Vec<Bounds<Pixels>> = cx
        .displays()
        .iter()
        .map(|display| display.bounds())
        .collect();
    let saved = store::load::<Option<SavedWindow>>(window_file);
    start_bounds(saved, &screens, default_bounds)
}

/// Where the window opens: as it was saved, or in the default place.
fn start_bounds(
    saved: Option<SavedWindow>,
    screens: &[Bounds<Pixels>],
    default_bounds: Bounds<Pixels>,
) -> WindowBounds {
    match saved {
        Some(saved) => saved.window_bounds(screens),
        None => WindowBounds::Windowed(default_bounds),
    }
}

fn window_options(window_bounds: WindowBounds) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(window_bounds),
        titlebar: Some(TitlebarOptions {
            title: Some("Naran".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(14.), px(14.))),
        }),
        ..Default::default()
    }
}

fn build_workspace(
    window: &mut Window,
    cx: &mut App,
    directory: &Path,
    window_file: &Path,
) -> Entity<Workspace> {
    let workspace = cx.new(|cx| Workspace::new(directory, cx));
    window.focus(&workspace.focus_handle(cx));
    save_window_on_change(&workspace, window_file, window, cx);
    save_window_on_close(window_file, window, cx);
    workspace
}

/// Saved on every move and resize, so a crash or a forced quit loses nothing.
fn save_window_on_change(
    workspace: &Entity<Workspace>,
    window_file: &Path,
    window: &mut Window,
    cx: &mut App,
) {
    let file = window_file.to_path_buf();
    workspace.update(cx, |_, cx| {
        cx.observe_window_bounds(window, move |_, window, _| save_window(&file, window))
            .detach();
    });
}

/// Saved once more when the window closes.
fn save_window_on_close(window_file: &Path, window: &mut Window, cx: &mut App) {
    let file = window_file.to_path_buf();
    window.on_window_should_close(cx, move |window, _| {
        save_window(&file, window);
        true
    });
}

fn quit_with_last_window(cx: &mut App) {
    cx.on_window_closed(|cx| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
}

#[cfg(test)]
mod tests;
