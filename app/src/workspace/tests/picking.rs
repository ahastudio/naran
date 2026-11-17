//! Picking several files and moving them together.

use super::*;

#[gpui::test]
fn shift_and_cmd_clicks_pick_files_and_one_action_moves_them_all(cx: &mut TestAppContext) {
    use gpui::Modifiers;
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);
    let untracked = state('?', '?', false);
    let click = |workspace: &Entity<Workspace>,
                 cx: &mut VisualTestContext,
                 name: &str,
                 modifiers: Modifiers| {
        workspace.update(cx, |workspace, cx| {
            workspace.click_file(
                ListedFile::new(name, Staging::Unstaged, untracked),
                modifiers,
                cx,
            );
        });
        cx.run_until_parked();
    };
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    let cmd = Modifiers {
        platform: true,
        ..Modifiers::default()
    };

    click(&workspace, cx, "a.txt", Modifiers::default());
    assert_eq!(
        picked(&workspace, cx),
        ["a.txt"],
        "a plain click picks one file"
    );
    click(&workspace, cx, "c.txt", shift);
    assert_eq!(
        picked(&workspace, cx),
        ["a.txt", "b.txt", "c.txt"],
        "Shift picks the range from the anchor"
    );
    click(&workspace, cx, "b.txt", cmd);
    assert_eq!(
        picked(&workspace, cx),
        ["a.txt", "c.txt"],
        "Cmd takes one file out"
    );
    click(&workspace, cx, "d.txt", cmd);
    assert_eq!(
        picked(&workspace, cx),
        ["a.txt", "c.txt", "d.txt"],
        "Cmd adds one file"
    );
    click(&workspace, cx, "b.txt", Modifiers::default());
    assert_eq!(
        picked(&workspace, cx),
        ["b.txt"],
        "a plain click starts again"
    );
    click(&workspace, cx, "a.txt", shift);
    assert_eq!(
        picked(&workspace, cx),
        ["a.txt", "b.txt"],
        "Shift also works upward"
    );

    // One action stages all the picked files, and nothing stays picked.
    workspace.update(cx, |workspace, cx| {
        let files = picked_paths(workspace);
        workspace.move_files(files, Staging::Unstaged, Staging::Staged, cx);
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx),
        [
            ("a.txt".to_string(), 'A', '.'),
            ("b.txt".to_string(), 'A', '.'),
            ("c.txt".to_string(), '?', '?'),
            ("d.txt".to_string(), '?', '?'),
        ]
    );
    assert_eq!(picked(&workspace, cx), [] as [String; 0]);
}

#[gpui::test]
fn shift_arrows_grow_the_picked_range_and_a_plain_arrow_picks_one(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.layout.pane = Pane::Files;
        workspace.step(1, cx);
    });
    cx.run_until_parked();
    assert_eq!(picked(&workspace, cx), ["a.txt"]);

    workspace.update(cx, |workspace, cx| workspace.extend(1, cx));
    workspace.update(cx, |workspace, cx| workspace.extend(1, cx));
    cx.run_until_parked();
    assert_eq!(
        picked(&workspace, cx),
        ["a.txt", "b.txt", "c.txt"],
        "Shift+Down twice"
    );
    workspace.update(cx, |workspace, cx| workspace.extend(-1, cx));
    cx.run_until_parked();
    assert_eq!(
        picked(&workspace, cx),
        ["a.txt", "b.txt"],
        "Shift+Up takes the last one back"
    );
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(
            workspace.shown.viewing.as_ref().unwrap().file,
            "b.txt",
            "the diff follows the newest row"
        );
    });

    workspace.update(cx, |workspace, cx| workspace.step(1, cx));
    cx.run_until_parked();
    assert_eq!(
        picked(&workspace, cx),
        ["c.txt"],
        "a plain arrow picks just the next file"
    );
}

#[gpui::test]
fn dropping_the_picked_files_moves_them_together(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.move_files(
            vec!["b.txt".into(), "d.txt".into()],
            Staging::Unstaged,
            Staging::Staged,
            cx,
        );
    });
    cx.run_until_parked();
    let mut now = changes(&workspace, cx);
    now.sort(); // git lists the staged files first; the order does not matter here
    assert_eq!(
        now,
        [
            ("a.txt".to_string(), '?', '?'),
            ("b.txt".to_string(), 'A', '.'),
            ("c.txt".to_string(), '?', '?'),
            ("d.txt".to_string(), 'A', '.'),
        ]
    );
    // And back, from the other list.
    workspace.update(cx, |workspace, cx| {
        workspace.move_files(
            vec!["b.txt".into(), "d.txt".into()],
            Staging::Staged,
            Staging::Unstaged,
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(
        changes(&workspace, cx)
            .iter()
            .filter(|(_, index, _)| *index == '?')
            .count(),
        4
    );
}
