//! The diff of a file and the pictures of an image.

use super::*;

#[gpui::test]
fn shows_the_diff_of_a_file_and_follows_it_through_staging(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("README.md"), "alpha\nmore\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "README.md".into(),
            Staging::Unstaged,
            state('.', 'M', false),
            cx,
        );
    });
    cx.run_until_parked();
    let lines = diff(&workspace, cx).unwrap();
    assert!(lines.contains(&"+more".to_string()), "{lines:?}");
    assert_eq!(viewing_staged(&workspace, cx), Some(false));

    workspace.update(cx, |workspace, cx| {
        workspace.stage(vec!["README.md".into()], cx);
    });
    cx.run_until_parked();
    let lines = diff(&workspace, cx).expect("still open after staging");
    assert!(
        lines.contains(&"+more".to_string()),
        "the staged diff is shown: {lines:?}"
    );
    assert_eq!(
        viewing_staged(&workspace, cx),
        Some(true),
        "the diff followed the file to Staged files"
    );

    workspace.update(cx, |workspace, cx| {
        workspace.unstage(vec!["README.md".into()], cx);
    });
    cx.run_until_parked();
    assert_eq!(
        viewing_staged(&workspace, cx),
        Some(false),
        "and back to Unstaged files"
    );

    git(&fixture.alpha, &["add", "README.md"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "more"]);
    workspace.update(cx, Workspace::refresh);
    cx.run_until_parked();
    assert_eq!(
        diff(&workspace, cx),
        None,
        "closed once the file is no longer changed"
    );
}

#[gpui::test]
fn shows_an_untracked_file_whole(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(fixture.beta.clone()), cx);
    });
    cx.run_until_parked();

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "draft.txt".into(),
            Staging::Unstaged,
            state('?', '?', false),
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(diff(&workspace, cx), Some(vec!["+draft".to_string()]));
}

#[gpui::test]
fn an_image_shows_before_on_the_left_and_after_on_the_right(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("logo.png"), TINY_PNG).unwrap();
    git(&fixture.alpha, &["add", "logo.png"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "logo"]);
    let mut changed = TINY_PNG.to_vec();
    changed.extend_from_slice(b"extra");
    std::fs::write(fixture.alpha.join("logo.png"), &changed).unwrap();
    std::fs::write(fixture.alpha.join("new.png"), TINY_PNG).unwrap();
    let (workspace, cx) = open(&fixture, cx);

    // Changed: both sides.
    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "logo.png".into(),
            Staging::Unstaged,
            state('.', 'M', false),
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(pictures(&workspace, cx), (true, true));
    workspace.read_with(cx, |workspace, _| {
        assert!(
            workspace.shown.viewing.as_ref().unwrap().lines.is_none(),
            "no text diff is read for an image"
        );
    });

    // New (untracked): nothing before.
    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "new.png".into(),
            Staging::Unstaged,
            state('?', '?', false),
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(pictures(&workspace, cx), (false, true));

    // Deleted: nothing after.
    std::fs::remove_file(fixture.alpha.join("logo.png")).unwrap();
    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "logo.png".into(),
            Staging::Unstaged,
            state('.', 'D', false),
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(pictures(&workspace, cx), (true, false));
}

#[gpui::test]
fn an_image_in_a_past_commit_has_no_before_when_it_was_added(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("logo.png"), TINY_PNG).unwrap();
    git(&fixture.alpha, &["add", "logo.png"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "add logo"]);
    let (workspace, cx) = open(&fixture, cx);
    let hash = workspace.read_with(cx, |workspace, _| {
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
    workspace.update(cx, |workspace, cx| workspace.pick_commit(hash, cx));
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        let inspecting = workspace.shown.inspecting.as_ref().unwrap();
        assert_eq!(inspecting.file.as_deref(), Some("logo.png"));
        let pictures = inspecting
            .pictures
            .as_ref()
            .expect("read")
            .as_ref()
            .unwrap();
        assert!(pictures.before.is_none() && pictures.after.is_some());
    });
}
