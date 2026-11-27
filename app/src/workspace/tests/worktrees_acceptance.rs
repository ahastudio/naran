//! Acceptance scenarios of worktrees, driven through the window the way a
//! person uses it: clicks, keys and edits made outside the app. The
//! worktrees are real linked worktrees of the fixture repositories.

use std::time::Duration;

use super::staging_acceptance::{click_row, drag, listed, named};
use super::*;

/// Adds a worktree on a new branch next to `repository`, named after it,
/// and returns its folder as Git prints it.
pub(super) fn add_worktree(repository: &Path, branch: &str) -> PathBuf {
    let name = repository.file_name().unwrap().to_string_lossy();
    let folder = repository
        .parent()
        .unwrap()
        .join(format!("{name}-{branch}"));
    let target = folder.to_string_lossy();
    git(
        repository,
        &["worktree", "add", "-q", "-b", branch, &target],
    );
    folder.canonicalize().unwrap()
}

/// The ids of the toggle of a repository and of the row of a worktree, which
/// are built from their folders.
pub(super) fn toggle(repository: &Path) -> &'static str {
    named(&format!("worktree-toggle-{}", repository.display()))
}

pub(super) fn row(worktree: &Path) -> &'static str {
    named(&format!("worktree-row-{}", worktree.display()))
}

fn worktrees_of(
    workspace: &Entity<Workspace>,
    repository: &Path,
    cx: &mut VisualTestContext,
) -> Vec<Entry> {
    workspace.read_with(cx, |workspace, _| {
        let entry = workspace.entries.iter().find(|e| e.path() == repository);
        entry.map(Entry::worktrees).unwrap_or_default()
    })
}

/// The rows on screen: bounds of rows drawn earlier stay in the test window,
/// so absence is read from the list the column is drawn from.
fn rows_on_screen(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Vec<PathBuf> {
    workspace.read_with(cx, |workspace, _| workspace.rows_on_screen())
}

fn is_open(workspace: &Entity<Workspace>, repository: &Path, cx: &mut VisualTestContext) -> bool {
    workspace.read_with(cx, |workspace, _| workspace.is_open(repository))
}

/// Lets the watcher deliver for up to five seconds, until `done` holds.
fn wait_until(cx: &mut VisualTestContext, mut done: impl FnMut(&mut VisualTestContext) -> bool) {
    for _ in 0..50 {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Opens the window with a worktree of alpha, opened and selected.
pub(super) fn open_selected_worktree<'a>(
    fixture: &Fixture,
    cx: &'a mut TestAppContext,
) -> (Entity<Workspace>, &'a mut VisualTestContext, PathBuf) {
    let folder = add_worktree(&fixture.alpha, "feature");
    std::fs::write(folder.join("wt.txt"), "work").unwrap();
    let (workspace, cx) = open(fixture, cx);
    click_row(cx, toggle(&fixture.alpha), Modifiers::none());
    click_row(cx, row(&folder), Modifiers::none());
    (workspace, cx, folder)
}

use gpui::Modifiers;

#[gpui::test]
fn worktrees_are_folded_until_the_toggle_is_clicked(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let one = add_worktree(&fixture.alpha, "one");
    let two = add_worktree(&fixture.alpha, "two");
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(worktrees_of(&workspace, &fixture.alpha, cx).len(), 2);
    assert_eq!(worktrees_of(&workspace, &fixture.beta, cx).len(), 0);
    assert_eq!(rows_on_screen(&workspace, cx).len(), 3);

    click_row(cx, toggle(&fixture.alpha), Modifiers::none());
    assert!(listed(cx, row(&one)));
    assert!(listed(cx, row(&two)));
    assert_eq!(rows_on_screen(&workspace, cx).len(), 5);

    click_row(cx, toggle(&fixture.alpha), Modifiers::none());
    assert!(!is_open(&workspace, &fixture.alpha, cx));
    assert_eq!(rows_on_screen(&workspace, cx).len(), 3);

    // Beta has no worktrees, so it has no toggle to click.
    assert!(!listed(cx, toggle(&fixture.beta)));
    assert!(!is_open(&workspace, &fixture.beta, cx));
}

#[gpui::test]
fn a_worktree_has_its_own_branch_and_state(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let folder = add_worktree(&fixture.alpha, "feature");
    std::fs::write(folder.join("wt.txt"), "work").unwrap();
    let (workspace, cx) = open(&fixture, cx);

    let worktrees = worktrees_of(&workspace, &fixture.alpha, cx);

    let Entry::Ready(summary) = &worktrees[0] else {
        panic!("the worktree is readable");
    };
    assert_eq!(summary.status.branch.as_deref(), Some("feature"));
    assert_eq!(summary.status.untracked, 1);
    assert_eq!(model::describe(&worktrees[0]).0, "Changes");
    workspace.read_with(cx, |workspace, _| {
        let alpha = &workspace.entries[0];
        assert_eq!(model::describe(alpha).0, "No upstream");
    });
}

#[gpui::test]
fn a_worktree_whose_folder_is_gone_is_shown_as_a_problem(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let folder = add_worktree(&fixture.alpha, "feature");
    std::fs::remove_dir_all(&folder).unwrap();
    let (workspace, cx) = open(&fixture, cx);

    let worktrees = worktrees_of(&workspace, &fixture.alpha, cx);

    assert_eq!(worktrees.len(), 1);
    assert!(matches!(worktrees[0], Entry::Failed { .. }));
    assert!(worktrees[0].has_problem());
    click_row(cx, toggle(&fixture.alpha), Modifiers::none());
    assert!(listed(cx, row(worktrees[0].path())));
}

#[gpui::test]
fn selecting_a_worktree_shows_the_changes_of_its_folder(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx, folder) = open_selected_worktree(&fixture, cx);

    assert_eq!(selected(&workspace, cx), Some(folder));
    let files = changes(&workspace, cx);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "wt.txt");
}

#[gpui::test]
fn staging_in_a_worktree_updates_its_row(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx, folder) = open_selected_worktree(&fixture, cx);

    drag(cx, "file-unstaged-wt.txt", "list-staged");

    assert_eq!(git_output(&folder, &["status", "--short"]), "A  wt.txt\n");
    let worktrees = worktrees_of(&workspace, &fixture.alpha, cx);
    let Entry::Ready(summary) = &worktrees[0] else {
        panic!("the worktree is readable");
    };
    assert_eq!((summary.status.staged, summary.status.untracked), (1, 0));
    assert_eq!(selected(&workspace, cx), Some(folder));
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.entries[0].change_count(), 0);
    });
}

#[gpui::test]
fn the_arrow_keys_pass_through_open_worktrees_only(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1])]);
    let folder = add_worktree(&fixture.alpha, "feature");
    let (workspace, cx) = open(&fixture, cx);

    cx.simulate_keystrokes("down");
    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));
    cx.simulate_keystrokes("up");

    click_row(cx, toggle(&fixture.alpha), Modifiers::none());
    cx.simulate_keystrokes("down");
    assert_eq!(selected(&workspace, cx), Some(folder));
    cx.simulate_keystrokes("down");
    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));
    cx.simulate_keystrokes("up up");
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));
}

#[gpui::test]
fn folding_a_repository_selects_it_when_its_worktree_was_selected(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx, _folder) = open_selected_worktree(&fixture, cx);

    click_row(cx, toggle(&fixture.alpha), Modifiers::none());

    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));
    assert!(!is_open(&workspace, &fixture.alpha, cx));
}

#[gpui::test]
fn a_removed_worktree_leaves_the_selection_on_a_repository(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx, folder) = open_selected_worktree(&fixture, cx);
    git(
        &fixture.alpha,
        &["worktree", "remove", "--force", &folder.to_string_lossy()],
    );

    cx.simulate_keystrokes("cmd-r");
    cx.run_until_parked();

    assert_eq!(worktrees_of(&workspace, &fixture.alpha, cx).len(), 0);
    assert!(!rows_on_screen(&workspace, cx).contains(&folder));
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));
}

#[gpui::test]
fn a_file_created_in_a_worktree_shows_without_a_refresh(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = fixture(&[]);
    let folder = add_worktree(&fixture.alpha, "feature");
    let (workspace, cx) = open(&fixture, cx);
    let untracked =
        |cx: &mut VisualTestContext| match worktrees_of(&workspace, &fixture.alpha, cx).first() {
            Some(Entry::Ready(summary)) => summary.status.untracked,
            _ => 0,
        };

    std::fs::write(folder.join("fresh.txt"), "x").unwrap();

    wait_until(cx, |cx| untracked(cx) == 1);
    assert_eq!(untracked(cx), 1);
}

#[gpui::test]
fn worktrees_are_listed_when_watching_is_off(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    add_worktree(&fixture.alpha, "feature");
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.services.watching = None;
        workspace.refresh(cx);
    });
    cx.run_until_parked();

    assert_eq!(worktrees_of(&workspace, &fixture.alpha, cx).len(), 1);
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.services.watching.is_none());
    });
}
