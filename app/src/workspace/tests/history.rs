//! The commit history: picking commits, loading older ones and going to HEAD.

use super::*;

#[gpui::test]
fn picking_a_commit_shows_its_files_and_diff_and_the_uncommitted_row_brings_the_lists_back(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("second.txt"), "two\n").unwrap();
    git(&fixture.alpha, &["add", "second.txt"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "second"]);
    std::fs::write(fixture.alpha.join("loose.txt"), "loose\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    let hash = workspace.read_with(cx, |workspace, _| {
        assert!(
            workspace.shown.inspecting.is_none(),
            "uncommitted changes are shown first"
        );
        workspace
            .shown
            .detail
            .as_ref()
            .unwrap()
            .history
            .as_ref()
            .unwrap()[0]
            .hash
            .clone()
    });

    workspace.update(cx, |workspace, cx| workspace.pick_commit(hash.clone(), cx));
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        let inspecting = workspace
            .shown
            .inspecting
            .as_ref()
            .expect("a commit is picked");
        let files: Vec<&str> = inspecting
            .files
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .iter()
            .map(|file| file.path.as_str())
            .collect();
        assert_eq!(files, ["second.txt"]);
        assert_eq!(
            inspecting.file.as_deref(),
            Some("second.txt"),
            "the first file opens by itself"
        );
        let lines = inspecting.lines.as_ref().unwrap().as_ref().unwrap();
        assert!(lines.contains(&"+two".to_string()), "{lines:?}");
        let detail = inspecting
            .detail
            .as_ref()
            .expect("the message is read")
            .as_ref()
            .unwrap();
        assert_eq!(detail.message, "second");
        assert_eq!(detail.hash, hash);
        assert_eq!(detail.parents.len(), 1);
        assert!(
            !detail.author.is_empty() && detail.author_email.contains('@'),
            "{detail:?}"
        );
        assert_eq!(
            detail.author_date.len(),
            "2026-11-16 21:05:00".len(),
            "{}",
            detail.author_date
        );
    });

    workspace.update(cx, Workspace::pick_uncommitted);
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.shown.inspecting.is_none());
    });
}

#[gpui::test]
fn a_clean_repository_starts_on_its_top_commit(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        let detail = workspace.shown.detail.as_ref().unwrap();
        assert!(
            detail.changes.as_ref().unwrap().is_empty(),
            "the fixture repository is clean"
        );
        let top = &detail.history.as_ref().unwrap()[0].hash;
        let inspecting = workspace
            .shown
            .inspecting
            .as_ref()
            .expect("the top commit is picked");
        assert_eq!(&inspecting.hash, top);
    });
}

#[gpui::test]
fn the_history_loads_older_commits_as_the_end_is_reached(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    add_commits(&fixture.alpha, 230);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();

    let counts = |workspace: &Entity<Workspace>, cx: &mut VisualTestContext| {
        workspace.read_with(cx, |workspace, _| {
            let detail = workspace.shown.detail.as_ref().unwrap();
            (
                detail.history.as_ref().unwrap().len(),
                detail.rows.len(),
                detail.more,
            )
        })
    };
    let (first, rows, more) = counts(&workspace, cx);
    assert_eq!((first, rows, more), (200, 200, true), "the first page");

    workspace.update(cx, Workspace::load_more_history);
    cx.run_until_parked();
    let (all, rows, more) = counts(&workspace, cx);
    assert!(all > 200 && all < 400, "the next page was added: {all}");
    assert_eq!(rows, all, "the graph covers every loaded commit");
    assert!(!more, "a short page means there is nothing older");

    // A refresh keeps the loaded length.
    workspace.update(cx, Workspace::refresh);
    cx.run_until_parked();
    assert_eq!(counts(&workspace, cx).0, all);
}

#[gpui::test]
fn go_to_head_finds_a_head_commit_that_is_older_than_the_loaded_page(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    // Another branch is 230 commits ahead, so the commit HEAD is on is far down the list.
    add_commits_to(&fixture.alpha, "side", 230);
    let head = git_output(&fixture.alpha, &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        let commits = workspace
            .shown
            .detail
            .as_ref()
            .unwrap()
            .history
            .as_ref()
            .unwrap();
        assert_eq!(commits.len(), 200, "only the first page is loaded");
        assert!(
            commits.iter().all(|commit| commit.hash != head),
            "HEAD is not in it"
        );
    });

    workspace.update(cx, Workspace::jump_to_head);
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        let commits = workspace
            .shown
            .detail
            .as_ref()
            .unwrap()
            .history
            .as_ref()
            .unwrap();
        assert!(
            commits.len() > 200,
            "older commits were read: {}",
            commits.len()
        );
        assert_eq!(
            workspace
                .shown
                .inspecting
                .as_ref()
                .map(|picked| picked.hash.as_str()),
            Some(head.as_str())
        );
        assert!(!workspace.progress.jumping_to_head);
    });
}

#[gpui::test]
fn go_to_head_picks_the_head_commit_when_it_is_already_loaded(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("x.txt"), "x\n").unwrap(); // uncommitted: the first row is Uncommitted changes
    let head = git_output(&fixture.alpha, &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, Workspace::jump_to_head);
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(
            workspace
                .shown
                .inspecting
                .as_ref()
                .map(|picked| picked.hash.as_str()),
            Some(head.as_str())
        );
    });
}
