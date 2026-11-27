//! Workspace behavior checked through GPUI's test context, with real Git
//! repositories in a temporary folder.

use std::path::{Path, PathBuf};
use std::process::Command;

use gpui::{Entity, TestAppContext, VisualTestContext};

use naran_core::{Change, Staging};

use gpui::{
    ExternalPaths, MouseButton, MouseDownEvent, Pixels, ScrollHandle, UniformListScrollHandle, px,
};

use super::detail::ListedFile;
use super::layout::{DIVIDER_WIDTH, Divider, Sizes};
use super::menu::{ContextMenu, MenuTarget};
use super::navigation::stepped;
use super::reading::forward_changes;
use super::state::{EditKind, Outcome, Pane, Pressed};
use super::*;
use crate::model;

mod adding;
mod context_menu;
mod controls;
mod diffs;
mod fixtures;
mod groups;
mod history;
mod keys;
mod layout;
mod notices;
mod picking;
mod repositories;
mod scrolling;
mod stack;
mod staging;
mod sticky_names;

pub(super) use fixtures::{Fixture, TINY_PNG, fixture, git, git_output};
use fixtures::{
    add_commits, add_commits_to, commit, fixture_with, four_untracked_files, many_lines_from,
    repository,
};

pub(super) fn open<'a>(
    fixture: &Fixture,
    cx: &'a mut TestAppContext,
) -> (Entity<Workspace>, &'a mut VisualTestContext) {
    cx.update(|cx| {
        crate::text_input::bind_keys(cx);
        bind_keys(cx);
    });
    let config = fixture.config.clone();
    let (workspace, cx) = cx.add_window_view(|window, cx| {
        let workspace = Workspace::new(&config, cx);
        window.focus(&workspace.focus_handle);
        workspace
    });
    cx.run_until_parked();
    (workspace, cx)
}

fn names(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Vec<String> {
    workspace.read_with(cx, |workspace, _| {
        workspace
            .members()
            .iter()
            .map(|entry| entry.name())
            .collect()
    })
}

fn selected(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Option<PathBuf> {
    workspace.read_with(cx, |workspace, _| workspace.selection.repository.clone())
}

fn changes(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Vec<(String, char, char)> {
    workspace.read_with(cx, |workspace, _| {
        let detail = workspace.shown.detail.as_ref().expect("details");
        detail
            .changes
            .as_ref()
            .unwrap()
            .iter()
            .map(|change| (change.path.clone(), change.index, change.worktree))
            .collect()
    })
}

fn state(index: char, worktree: char, conflicted: bool) -> model::FileState {
    model::FileState::of(&Change {
        path: String::new(),
        original: None,
        index,
        worktree,
        conflicted,
    })
}

/// The open diff, or `None` when no diff is open.
fn diff(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Option<Vec<String>> {
    workspace.read_with(cx, |workspace, _| {
        workspace
            .shown
            .viewing
            .as_ref()
            .map(|viewing| viewing.lines.clone().expect("loaded").expect("readable"))
    })
}

fn viewing_staged(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Option<bool> {
    workspace.read_with(cx, |workspace, _| {
        workspace
            .shown
            .viewing
            .as_ref()
            .map(|viewing| viewing.staging == Staging::Staged)
    })
}

fn right_click(cx: &mut VisualTestContext, x: f32, y: f32) {
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Right,
        position: gpui::point(px(x), px(y)),
        modifiers: gpui::Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    cx.run_until_parked();
}

fn pictures(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> (bool, bool) {
    workspace.read_with(cx, |workspace, _| {
        let pictures = workspace
            .shown
            .viewing
            .as_ref()
            .unwrap()
            .pictures
            .as_ref()
            .expect("read")
            .as_ref()
            .unwrap();
        (pictures.before.is_some(), pictures.after.is_some())
    })
}

fn picked(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Vec<String> {
    workspace.read_with(cx, |workspace, _| {
        workspace.shown.marked.paths.iter().cloned().collect()
    })
}

fn stack(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> Option<Vec<detail::StackRow>> {
    workspace.read_with(cx, |workspace, _| {
        workspace
            .shown
            .stacked
            .as_ref()
            .and_then(|stacked| stacked.rows.as_ref())
            .map(|rows| rows.as_ref().unwrap().clone())
    })
}

/// How far a list has been scrolled down, in pixels.
fn scrolled(handle: &UniformListScrollHandle) -> f32 {
    -f32::from(handle.0.borrow().base_handle.offset().y)
}

/// Clicks the link or button with this id, found where it was last drawn.
fn click(cx: &mut VisualTestContext, id: &'static str) {
    let bounds = cx
        .debug_bounds(id)
        .unwrap_or_else(|| panic!("{id} is on screen"));
    cx.simulate_click(bounds.center(), gpui::Modifiers::default());
    cx.run_until_parked();
}

fn picked_paths(workspace: &Workspace) -> Vec<String> {
    let mut files: Vec<String> = workspace.shown.marked.paths.iter().cloned().collect();
    files.sort();
    files
}
mod commit_history_acceptance;
mod diff_viewing_acceptance;
mod diff_viewing_files_acceptance;
mod group_order_acceptance;
mod groups_and_repositories_acceptance;
mod layout_acceptance;
mod live_updates_acceptance;
mod moving_a_repository_to_another_group_acceptance;
mod renamed_files_acceptance;
mod staging_acceptance;
mod staging_diff_acceptance;
mod worktrees_acceptance;
mod worktrees_detail_acceptance;
