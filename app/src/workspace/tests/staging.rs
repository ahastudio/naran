//! Staging and unstaging files, hunks and lines.

use super::*;

/// Puts a commit that writes its own name into `README.md` on `branch`, which
/// starts from `main`, and goes back to `main`.
fn commit_on(path: &Path, branch: &str) {
    git(path, &["switch", "-q", "-C", branch]);
    commit(path, "README.md", branch, branch);
    git(path, &["switch", "-q", "main"]);
}

#[gpui::test]
fn stages_and_unstages_whole_files(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(fixture.beta.clone()), cx);
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx),
        [("draft.txt".to_string(), '?', '?')]
    );

    workspace.update(cx, |workspace, cx| {
        workspace.stage(vec!["draft.txt".into()], cx);
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx),
        [("draft.txt".to_string(), 'A', '.')]
    );

    workspace.update(cx, |workspace, cx| {
        workspace.unstage(vec!["draft.txt".into()], cx);
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx),
        [("draft.txt".to_string(), '?', '?')]
    );
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(
            workspace.feedback.message.clone().unwrap().1,
            Outcome::Succeeded,
            "the last operation succeeded"
        );
    });
}

#[gpui::test]
fn stages_only_the_picked_lines_and_unstages_them_again(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("notes.txt"), "l1\nl2\nl3\n").unwrap();
    git(&fixture.alpha, &["add", "notes.txt"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "notes"]);
    std::fs::write(fixture.alpha.join("notes.txt"), "l1\nL2\nl3\nl4\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "notes.txt".into(),
            Staging::Unstaged,
            state('.', 'M', false),
            cx,
        );
    });
    cx.run_until_parked();
    let lines = diff(&workspace, cx).unwrap();
    let added = lines
        .iter()
        .position(|line| line == "+l4")
        .expect("the added line");
    workspace.update(cx, |workspace, cx| {
        workspace.toggle_pick(added, cx);
        let picked: Vec<usize> = workspace
            .shown
            .viewing
            .as_ref()
            .unwrap()
            .picked
            .iter()
            .copied()
            .collect();
        assert_eq!(picked, [added]);
        workspace.apply_lines(picked, cx);
    });
    cx.run_until_parked();

    let staged = git_output(&fixture.alpha, &["diff", "--cached", "--", "notes.txt"]);
    assert!(staged.contains("+l4"), "{staged}");
    assert!(
        !staged.contains("+L2"),
        "only the picked line is staged: {staged}"
    );
    assert_eq!(
        viewing_staged(&workspace, cx),
        Some(false),
        "the rest of the file stays in Unstaged files"
    );

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "notes.txt".into(),
            Staging::Staged,
            state('M', 'M', false),
            cx,
        );
    });
    cx.run_until_parked();
    let lines = diff(&workspace, cx).unwrap();
    let added = lines
        .iter()
        .position(|line| line == "+l4")
        .expect("the staged line");
    workspace.update(cx, |workspace, cx| workspace.apply_lines(vec![added], cx));
    cx.run_until_parked();
    let staged = git_output(&fixture.alpha, &["diff", "--cached", "--", "notes.txt"]);
    assert_eq!(staged, "", "nothing is staged after unstaging the line");
}

#[gpui::test]
fn stages_a_whole_hunk(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let before = many_lines_from(1, 30);
    std::fs::write(fixture.alpha.join("long.txt"), &before).unwrap();
    git(&fixture.alpha, &["add", "long.txt"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "long"]);
    std::fs::write(
        fixture.alpha.join("long.txt"),
        before
            .replace("line 2\n", "top\n")
            .replace("line 29\n", "bottom\n"),
    )
    .unwrap();
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "long.txt".into(),
            Staging::Unstaged,
            state('.', 'M', false),
            cx,
        );
    });
    cx.run_until_parked();
    let lines = diff(&workspace, cx).unwrap();
    let hunks = model::hunks(&lines);
    assert_eq!(hunks.len(), 2, "{lines:?}");
    let first = model::changed_lines(&lines, hunks[0].clone());
    workspace.update(cx, |workspace, cx| workspace.apply_lines(first, cx));
    cx.run_until_parked();

    let staged = git_output(&fixture.alpha, &["diff", "--cached", "--", "long.txt"]);
    assert!(
        staged.contains("+top") && !staged.contains("+bottom"),
        "{staged}"
    );
}

#[gpui::test]
fn a_conflicted_file_can_only_be_staged_whole(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    conflict(&fixture);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.open_file(
            "README.md".into(),
            Staging::Unstaged,
            state('U', 'U', true),
            cx,
        );
    });
    cx.run_until_parked();
    let all: Vec<usize> = (0..diff(&workspace, cx).unwrap().len()).collect();
    workspace.update(cx, |workspace, cx| workspace.apply_lines(all, cx));
    cx.run_until_parked();
    let unmerged = git_output(&fixture.alpha, &["ls-files", "--unmerged"]);
    assert!(
        !unmerged.is_empty(),
        "partial staging did nothing to the conflict"
    );
}

/// Leaves `alpha` stopped in a merge with `README.md` conflicted.
fn conflict(fixture: &Fixture) {
    let path = &fixture.alpha;
    for branch in ["topic", "main"] {
        commit_on(path, branch);
    }
    let merge = Command::new("git")
        .args(["merge", "-q", "topic"])
        .current_dir(path)
        .output();
    assert!(
        !merge.unwrap().status.success(),
        "the merge must stop on a conflict"
    );
}

#[gpui::test]
fn dropping_a_file_on_the_other_list_stages_or_unstages_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(fixture.beta.clone()), cx);
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx),
        [("draft.txt".to_string(), '?', '?')]
    );

    // Dropped on its own list: nothing happens.
    workspace.update(cx, |workspace, cx| {
        workspace.move_files(
            vec!["draft.txt".into()],
            Staging::Unstaged,
            Staging::Unstaged,
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx),
        [("draft.txt".to_string(), '?', '?')]
    );

    workspace.update(cx, |workspace, cx| {
        workspace.move_files(
            vec!["draft.txt".into()],
            Staging::Unstaged,
            Staging::Staged,
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx),
        [("draft.txt".to_string(), 'A', '.')]
    );

    workspace.update(cx, |workspace, cx| {
        workspace.move_files(
            vec!["draft.txt".into()],
            Staging::Staged,
            Staging::Unstaged,
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx),
        [("draft.txt".to_string(), '?', '?')]
    );
}
