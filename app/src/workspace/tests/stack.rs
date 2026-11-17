//! The stack of diffs shown when several files are picked.

use super::*;

#[gpui::test]
fn picking_several_files_shows_all_their_diffs_one_after_another(cx: &mut TestAppContext) {
    use super::detail::StackRow;
    use gpui::Modifiers;
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);
    let untracked = state('?', '?', false);
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
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

    // One file: the usual single diff, nothing stacked.
    click(&workspace, cx, "a.txt", Modifiers::default());
    assert!(stack(&workspace, cx).is_none());

    // Three files: their diffs, each under the head of its file.
    click(&workspace, cx, "c.txt", shift);
    let rows = stack(&workspace, cx).expect("the picked files are stacked");
    let heads: Vec<&str> = rows
        .iter()
        .filter_map(|row| match row {
            StackRow::File { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(heads, ["a.txt", "b.txt", "c.txt"]);
    for name in ["a.txt", "b.txt", "c.txt"] {
        assert!(
            rows.iter()
                .any(|row| matches!(row, StackRow::Line(line) if line.contains(name))),
            "{name}"
        );
    }
    assert!(
        !rows
            .iter()
            .any(|row| matches!(row, StackRow::Line(line) if line.contains("d.txt")))
    );
    assert!(
        matches!(rows[0], StackRow::File { badge: '?', .. }),
        "the head carries the state badge"
    );

    // Back to one file: the stack goes away.
    click(&workspace, cx, "b.txt", Modifiers::default());
    assert!(stack(&workspace, cx).is_none());

    // Staging the picked files clears it too.
    click(&workspace, cx, "a.txt", shift);
    assert!(stack(&workspace, cx).is_some());
    workspace.update(cx, |workspace, cx| {
        let files = picked_paths(workspace);
        workspace.move_files(files, Staging::Unstaged, Staging::Staged, cx);
    });
    cx.run_until_parked();
    assert!(stack(&workspace, cx).is_none());
}

#[gpui::test]
fn a_past_commit_hides_the_stack_and_coming_back_shows_it_again(cx: &mut TestAppContext) {
    use gpui::Modifiers;
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);
    let untracked = state('?', '?', false);
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    workspace.update(cx, |workspace, cx| {
        workspace.click_file(
            ListedFile::new("a.txt", Staging::Unstaged, untracked),
            Modifiers::default(),
            cx,
        );
    });
    workspace.update(cx, |workspace, cx| {
        workspace.click_file(
            ListedFile::new("b.txt", Staging::Unstaged, untracked),
            shift,
            cx,
        );
    });
    cx.run_until_parked();
    assert!(stack(&workspace, cx).is_some());

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
    assert!(
        workspace.read_with(cx, |workspace, _| workspace.shown.stacked.is_none()),
        "a past commit shows its own diff"
    );

    workspace.update(cx, Workspace::pick_uncommitted);
    cx.run_until_parked();
    assert!(
        stack(&workspace, cx).is_some(),
        "the picked files are stacked again"
    );
}

#[gpui::test]
fn the_stack_is_limited_so_a_huge_pick_stays_light(cx: &mut TestAppContext) {
    use super::detail::StackRow;
    use gpui::Modifiers;
    let fixture = fixture(&[]);
    for index in 0..45 {
        std::fs::write(
            fixture.alpha.join(format!("f{index:02}.txt")),
            "x\n".repeat(600),
        )
        .unwrap();
    }
    let (workspace, cx) = open(&fixture, cx);
    let untracked = state('?', '?', false);
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    workspace.update(cx, |workspace, cx| {
        workspace.click_file(
            ListedFile::new("f00.txt", Staging::Unstaged, untracked),
            Modifiers::default(),
            cx,
        );
    });
    workspace.update(cx, |workspace, cx| {
        workspace.click_file(
            ListedFile::new("f44.txt", Staging::Unstaged, untracked),
            shift,
            cx,
        );
    });
    cx.run_until_parked();
    let rows = stack(&workspace, cx).unwrap();
    let heads = rows
        .iter()
        .filter(|row| matches!(row, StackRow::File { .. }))
        .count();
    assert_eq!(heads, 40, "at most 40 files are read");
    assert!(
        matches!(rows.last(), Some(StackRow::Note(note)) if note.contains("5 more files")),
        "{:?}",
        rows.last()
    );
    assert!(
        rows.iter().any(
            |row| matches!(row, StackRow::Note(note) if note.contains("more lines in this file"))
        ),
        "long files are cut"
    );
    assert!(
        rows.len() < 40 * 510,
        "the whole stack stays bounded: {}",
        rows.len()
    );
}

#[gpui::test]
fn the_files_of_a_commit_are_picked_with_shift_cmd_and_cmd_a_and_stacked(cx: &mut TestAppContext) {
    use super::detail::StackRow;
    use gpui::Modifiers;
    let fixture = fixture(&[]);
    for file in ["a.txt", "b.txt", "c.txt", "d.txt"] {
        std::fs::write(fixture.alpha.join(file), format!("{file}\n")).unwrap();
    }
    git(&fixture.alpha, &["add", "."]);
    git(&fixture.alpha, &["commit", "-q", "-m", "four files"]);
    let (workspace, cx) = open(&fixture, cx);
    let hash = workspace.read_with(cx, |workspace, _| {
        let detail = workspace.shown.detail.as_ref().unwrap();
        detail.history.as_ref().unwrap()[0].hash.clone()
    });
    workspace.update(cx, |workspace, cx| workspace.pick_commit(hash, cx));
    cx.run_until_parked();
    let picked = |cx: &mut VisualTestContext| {
        workspace.read_with(cx, |workspace, _| {
            let inspecting = workspace.shown.inspecting.as_ref().unwrap();
            inspecting.picked.iter().cloned().collect::<Vec<_>>()
        })
    };
    let click = |cx: &mut VisualTestContext, file: &str, modifiers: Modifiers| {
        workspace.update(cx, |workspace, cx| {
            workspace.click_commit_file(file.into(), modifiers, cx);
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

    click(cx, "b.txt", Modifiers::default());
    assert_eq!(picked(cx), ["b.txt"]);
    click(cx, "d.txt", shift);
    assert_eq!(picked(cx), ["b.txt", "c.txt", "d.txt"]);
    click(cx, "d.txt", shift);
    assert_eq!(
        picked(cx),
        ["b.txt", "c.txt", "d.txt"],
        "the same range again"
    );
    click(cx, "c.txt", cmd);
    assert_eq!(picked(cx), ["b.txt", "d.txt"]);
    let stacked_paths = workspace.read_with(cx, |workspace, _| {
        let stacked = workspace.shown.stacked.as_ref().unwrap();
        let rows = stacked.rows.as_ref().unwrap().as_ref().unwrap();
        rows.iter()
            .filter_map(|row| match row {
                StackRow::File { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(stacked_paths, ["b.txt", "d.txt"]);
    click(cx, "c.txt", cmd);
    assert_eq!(
        picked(cx),
        ["b.txt", "c.txt", "d.txt"],
        "Cmd adds a file too"
    );

    workspace.update(cx, |workspace, _| workspace.focus_pane(Pane::Files));
    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();
    assert_eq!(picked(cx), ["a.txt", "b.txt", "c.txt", "d.txt"]);

    workspace.update(cx, |workspace, cx| workspace.step(1, cx));
    cx.run_until_parked();
    assert_eq!(picked(cx).len(), 1, "a plain arrow picks one");
    assert!(workspace.read_with(cx, |workspace, _| workspace.shown.stacked.is_none()));
    workspace.update(cx, |workspace, cx| workspace.extend(-1, cx));
    cx.run_until_parked();
    assert_eq!(picked(cx).len(), 2, "Shift with Up grows the picks");
}
