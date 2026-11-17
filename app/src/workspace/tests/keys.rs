//! Keys that move, pick and copy.

use super::*;

#[test]
fn stepping_stays_inside_the_list() {
    assert_eq!(stepped(0, None, 1), None);
    assert_eq!(stepped(3, None, 1), Some(0));
    assert_eq!(stepped(3, None, -1), Some(2));
    assert_eq!(stepped(3, Some(0), -1), Some(0));
    assert_eq!(stepped(3, Some(1), 1), Some(2));
    assert_eq!(stepped(3, Some(2), 1), Some(2));
}

#[gpui::test]
fn arrow_keys_move_through_the_list_that_was_clicked_last(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("second.txt"), "two\n").unwrap();
    git(&fixture.alpha, &["add", "second.txt"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "second"]);
    std::fs::write(fixture.alpha.join("loose.txt"), "loose\n").unwrap();
    std::fs::write(fixture.alpha.join("other.txt"), "other\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();

    // Files: Unstaged files are the only ones, so down opens the first.
    workspace.update(cx, |workspace, cx| {
        workspace.layout.pane = Pane::Files;
        workspace.step(1, cx);
    });
    cx.run_until_parked();
    let first = workspace.read_with(cx, |workspace, _| {
        workspace.shown.viewing.as_ref().unwrap().file.clone()
    });
    workspace.update(cx, |workspace, cx| workspace.step(1, cx));
    cx.run_until_parked();
    let second = workspace.read_with(cx, |workspace, _| {
        workspace.shown.viewing.as_ref().unwrap().file.clone()
    });
    assert_ne!(first, second, "down moves to the next file");
    workspace.update(cx, |workspace, cx| workspace.step(-1, cx));
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.shown.viewing.as_ref().unwrap().file, first);
    });

    // Commits: down from the Uncommitted changes row picks the top commit.
    workspace.update(cx, |workspace, cx| {
        workspace.layout.pane = Pane::Commits;
        workspace.step(1, cx);
    });
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.shown.inspecting.is_some());
    });
    workspace.update(cx, |workspace, cx| workspace.step(-1, cx));
    workspace.read_with(cx, |workspace, _| {
        assert!(
            workspace.shown.inspecting.is_none(),
            "up returns to Uncommitted changes"
        );
    });
}

#[gpui::test]
fn cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);
    let untracked = state('?', '?', false);
    workspace.update(cx, |workspace, cx| {
        workspace.focus_pane(Pane::Files);
        workspace.click_file(
            ListedFile::new("a.txt", Staging::Unstaged, untracked),
            gpui::Modifiers::default(),
            cx,
        );
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("select-all-false").is_none(),
        "no Select all button"
    );

    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();
    let picked = workspace.read_with(cx, |workspace, _| {
        workspace
            .shown
            .marked
            .paths
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    });
    assert_eq!(picked, ["a.txt", "b.txt", "c.txt", "d.txt"]);

    // After pressing the diff, Cmd+A picks its text and leaves the files alone.
    workspace.update(cx, |workspace, cx| {
        workspace.click_file(
            ListedFile::new("a.txt", Staging::Unstaged, untracked),
            gpui::Modifiers::default(),
            cx,
        );
    });
    cx.run_until_parked();
    let area = cx
        .debug_bounds("diff-lines")
        .expect("the diff of a.txt is shown");
    let first_line = area.origin + gpui::point(px(80.), px(8.));
    cx.simulate_click(first_line, gpui::Modifiers::default());
    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.interaction.pressed, Pressed::Text);
        assert_eq!(
            workspace.shown.marked.paths.len(),
            1,
            "the files were not picked"
        );
        assert!(
            workspace.texts.copied().is_some(),
            "the text of the diff is picked"
        );
    });
}

#[gpui::test]
fn cmd_a_on_the_files_of_a_past_commit_does_not_pick_the_text_below(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("one.txt"), "one\n").unwrap();
    std::fs::write(fixture.alpha.join("two.txt"), "two\n").unwrap();
    git(&fixture.alpha, &["add", "-A"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "two files"]);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked(); // a clean repository starts on its top commit

    // Pressing the file list and Cmd+A picks no text.
    let files = cx
        .debug_bounds("commit-files-list")
        .expect("the files of the commit are shown");
    cx.simulate_click(
        files.origin + gpui::point(px(40.), px(10.)),
        gpui::Modifiers::default(),
    );
    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.interaction.pressed, Pressed::List);
        assert_eq!(
            workspace.texts.copied(),
            None,
            "the commit information is not picked"
        );
    });

    // Pressing the commit information and Cmd+A picks its text.
    let info = cx
        .debug_bounds("commit-info")
        .expect("the commit information is shown");
    cx.simulate_click(
        info.origin + gpui::point(px(40.), px(12.)),
        gpui::Modifiers::default(),
    );
    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.interaction.pressed, Pressed::Text);
        let copied = workspace.texts.copied().expect("its text is picked");
        assert!(copied.contains("two files"), "{copied}");
    });
}

#[gpui::test]
fn the_window_asks_the_workspace_for_its_focus_handle(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.read_with(cx, |workspace, cx| {
        let handle = Focusable::focus_handle(workspace, cx);
        assert_eq!(handle, workspace.focus_handle);
    });
}

#[gpui::test]
fn copying_puts_the_selected_text_on_the_clipboard_and_nothing_else_when_none_is_selected(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    let clipboard =
        |cx: &mut VisualTestContext| cx.read_from_clipboard().and_then(|item| item.text());

    workspace.update(cx, |workspace, cx| {
        workspace.texts.info.set_lines(vec!["hello".into()]);
        workspace.texts.select_all();
        workspace.copy_text(cx);
    });
    assert_eq!(clipboard(cx).as_deref(), Some("hello"));

    workspace.update(cx, |workspace, cx| {
        workspace.texts.info.set_lines(vec!["other".into()]);
        workspace.copy_text(cx);
    });
    assert_eq!(clipboard(cx).as_deref(), Some("hello"));
}
