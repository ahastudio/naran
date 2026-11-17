//! Acceptance scenarios of live updates, driven through the window the way a
//! person uses it: keys, clicks, and edits made outside the app.

use std::time::Duration;

use super::*;

fn state_of(workspace: &Entity<Workspace>, path: &Path, cx: &mut VisualTestContext) -> String {
    workspace.read_with(cx, |workspace, _| {
        let entry = workspace.entries.iter().find(|entry| entry.path() == path);
        entry
            .map(|entry| model::describe(entry).0)
            .unwrap_or_default()
    })
}

fn untracked_of(workspace: &Entity<Workspace>, path: &Path, cx: &mut VisualTestContext) -> u32 {
    workspace.read_with(cx, |workspace, _| {
        match workspace.entries.iter().find(|entry| entry.path() == path) {
            Some(Entry::Ready(summary)) => summary.status.untracked,
            _ => 0,
        }
    })
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

#[gpui::test]
fn a_burst_of_writes_ends_with_every_file_counted(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    for number in 0..10 {
        std::fs::write(fixture.gamma.join(format!("f{number}.txt")), "x").unwrap();
    }

    wait_until(cx, |cx| untracked_of(&workspace, &fixture.gamma, cx) == 10);
    assert_eq!(untracked_of(&workspace, &fixture.gamma, cx), 10);
    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "Changes");
}

#[gpui::test]
fn ignored_files_are_not_counted_but_the_next_real_change_is(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = fixture(&[]);
    commit(&fixture.gamma, ".gitignore", "*.log\n", "ignore logs");
    let (workspace, cx) = open(&fixture, cx);

    std::fs::write(fixture.gamma.join("build.log"), "noise").unwrap();
    std::fs::write(fixture.gamma.join("real.txt"), "work").unwrap();

    wait_until(cx, |cx| untracked_of(&workspace, &fixture.gamma, cx) == 1);
    assert_eq!(untracked_of(&workspace, &fixture.gamma, cx), 1);
}

#[gpui::test]
fn the_refresh_button_waits_while_reading_and_returns_after(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, Workspace::refresh);
    assert!(workspace.read_with(cx, |workspace, _| workspace.progress.reading));

    cx.run_until_parked();
    assert!(!workspace.read_with(cx, |workspace, _| workspace.progress.reading));
}

#[gpui::test]
fn the_state_shows_the_most_urgent_word_after_a_refresh(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(state_of(&workspace, &fixture.beta, cx), "Changes");
    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "No upstream");

    conflicting_merge(&fixture.gamma);
    cx.simulate_keystrokes("cmd-r");
    cx.run_until_parked();

    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "Merging");
}

/// Leaves `path` in the middle of a merge that conflicts.
fn conflicting_merge(path: &Path) {
    git(path, &["checkout", "-q", "-b", "other"]);
    commit(path, "README.md", "other\n", "other");
    git(path, &["checkout", "-q", "main"]);
    commit(path, "README.md", "main\n", "main");
    let merged = Command::new("git")
        .args(["merge", "other"])
        .current_dir(path)
        .output()
        .unwrap();
    assert!(!merged.status.success());
}

#[gpui::test]
fn a_repository_that_cannot_be_read_shows_its_error_as_a_problem(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    std::fs::remove_dir_all(&fixture.alpha).unwrap();
    cx.simulate_keystrokes("cmd-r");
    cx.run_until_parked();

    workspace.read_with(cx, |workspace, _| {
        let alpha = &workspace.entries[0];
        assert!(matches!(alpha, Entry::Failed { .. }));
        assert_eq!(model::describe(alpha).1, model::Tone::Problem);
        assert!(
            workspace.entries[1..]
                .iter()
                .all(|entry| matches!(entry, Entry::Ready(_)))
        );
    });
}

fn refresh_with_keys(cx: &mut VisualTestContext) {
    cx.simulate_keystrokes("cmd-r");
    cx.run_until_parked();
}

/// Puts `path` on a branch that tracks `main`.
fn track_main(path: &Path) {
    git(
        path,
        &["checkout", "-q", "-b", "feature", "--track", "main"],
    );
}

fn commit_on(path: &Path, branch: &str, file: &str) {
    git(path, &["checkout", "-q", branch]);
    commit(path, file, file, file);
    git(path, &["checkout", "-q", "feature"]);
}

#[gpui::test]
fn behind_then_diverged_follow_the_upstream_after_refreshes(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    track_main(&fixture.gamma);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "Clean");

    commit_on(&fixture.gamma, "main", "upstream.txt");
    refresh_with_keys(cx);
    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "Behind");

    commit(&fixture.gamma, "local.txt", "local", "local");
    refresh_with_keys(cx);
    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "Diverged");
}

#[gpui::test]
fn ahead_shows_after_a_local_commit_and_changes_outrank_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    track_main(&fixture.gamma);
    let (workspace, cx) = open(&fixture, cx);

    commit(&fixture.gamma, "local.txt", "local", "local");
    refresh_with_keys(cx);
    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "Ahead");

    std::fs::write(fixture.gamma.join("draft.txt"), "x").unwrap();
    refresh_with_keys(cx);
    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "Changes");
}

#[gpui::test]
fn refreshing_twice_in_a_row_reads_the_latest_state_each_time(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    std::fs::write(fixture.gamma.join("one.txt"), "1").unwrap();
    refresh_with_keys(cx);
    assert_eq!(untracked_of(&workspace, &fixture.gamma, cx), 1);

    std::fs::write(fixture.gamma.join("two.txt"), "2").unwrap();
    refresh_with_keys(cx);
    assert_eq!(untracked_of(&workspace, &fixture.gamma, cx), 2);
}

#[gpui::test]
fn a_group_without_repositories_has_nothing_to_read(cx: &mut TestAppContext) {
    let fixture = fixture(&[("empty", &[])]);
    let (workspace, cx) = open(&fixture, cx);

    refresh_with_keys(cx);

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.entries.len(), 0);
        assert!(!workspace.progress.reading);
    });
}

#[gpui::test]
fn a_registered_folder_that_is_gone_is_listed_as_failed_and_the_rest_still_work(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    std::fs::remove_dir_all(&fixture.alpha).unwrap();
    let (workspace, cx) = open(&fixture, cx);

    workspace.read_with(cx, |workspace, _| {
        assert!(matches!(workspace.entries[0], Entry::Failed { .. }));
    });
    assert_eq!(state_of(&workspace, &fixture.beta, cx), "Changes");
    assert_eq!(state_of(&workspace, &fixture.gamma, cx), "No upstream");
}
