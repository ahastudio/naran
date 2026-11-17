//! The diff of the open file: its colors, files that cannot be shown as
//! text, pictures, and picking lines and hunks to stage.

use gpui::{MouseButton, point, px};

use super::*;
use crate::model::LineKind;
use crate::theme::Theme;
use crate::workspace::detail::diff_rows::DIFF_LIMIT;
use crate::workspace::detail::stack_view::{kind_background, kind_color, visible_text};
use crate::workspace::detail::versions::{UNTRACKED_LIMIT, read_untracked};
use crate::workspace::tests::{TINY_PNG, git, git_output};

#[test]
fn each_kind_of_line_has_its_colors() {
    let theme = Theme::dark();
    assert_eq!(
        kind_background(LineKind::Added, &theme),
        theme.added.opacity(0.14)
    );
    assert_eq!(
        kind_background(LineKind::Removed, &theme),
        theme.removed.opacity(0.14)
    );
    assert_eq!(
        kind_background(LineKind::Hunk, &theme),
        theme.surface.opacity(0.6)
    );
    for kind in [LineKind::Meta, LineKind::Context] {
        assert_eq!(kind_background(kind, &theme), gpui::transparent_black());
    }
    for kind in [LineKind::Hunk, LineKind::Meta] {
        assert_eq!(kind_color(kind, &theme), theme.muted);
    }
    for kind in [LineKind::Added, LineKind::Removed, LineKind::Context] {
        assert_eq!(kind_color(kind, &theme), theme.text);
    }
    assert_eq!(visible_text(""), " ");
    assert_eq!(visible_text("a\tb"), "a    b");
}

#[test]
fn an_untracked_file_that_is_big_or_not_text_is_told_so_instead_of_shown() {
    let directory = tempfile::tempdir().unwrap();
    let big = directory.path().join("big.txt");
    let size = usize::try_from(UNTRACKED_LIMIT).unwrap() + 1;
    std::fs::write(&big, vec![b'x'; size]).unwrap();
    let binary = directory.path().join("binary.bin");
    std::fs::write(&binary, [0xff, 0xfe, 0x00]).unwrap();

    assert!(read_untracked(&big)[0].starts_with("The file is too large to show"));
    assert_eq!(read_untracked(&binary), ["The file is not text."]);
}

#[gpui::test]
fn a_diff_longer_than_the_limit_is_cut_with_a_note(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let lines: Vec<String> = (0..DIFF_LIMIT + 5)
        .map(|number| format!("line {number}\n"))
        .collect();
    std::fs::write(fixture.alpha.join("long.txt"), lines.concat()).unwrap();
    let (workspace, cx) = open(&fixture, cx);

    update(&workspace, cx, |workspace, cx| {
        let state = FileState::of(&untracked("long.txt"));
        workspace.open_file("long.txt".into(), Staging::Unstaged, state, cx);
    });

    assert!(cx.debug_bounds("omitted-note").is_some());
}

#[gpui::test]
fn a_staged_image_shows_the_committed_picture_before_and_the_staged_one_after(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("pic.png"), TINY_PNG).unwrap();
    git(&fixture.alpha, &["add", "pic.png"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "picture"]);
    let mut changed = TINY_PNG.to_vec();
    changed.extend([0, 0]);
    std::fs::write(fixture.alpha.join("pic.png"), &changed).unwrap();
    git(&fixture.alpha, &["add", "pic.png"]);
    let (workspace, cx) = open(&fixture, cx);
    let state = FileState {
        kind: FileKind::Tracked,
        staged: true,
        unstaged: false,
    };

    update(&workspace, cx, |workspace, cx| {
        workspace.open_file("pic.png".into(), Staging::Staged, state, cx);
    });

    workspace.read_with(cx, |workspace, _| {
        let viewing = workspace.shown.viewing.as_ref().unwrap();
        let pictures = viewing.pictures.as_ref().unwrap().as_ref().unwrap();
        assert!(pictures.before.is_some() && pictures.after.is_some());
    });
}

#[gpui::test]
fn clicking_a_changed_line_picks_it_for_staging(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("README.md"), "alpha\nsecond\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    let row = cx.debug_bounds("file-unstaged-README.md").unwrap();
    click_at(cx, row.center());

    let rows = (0..8).filter_map(|index| {
        let name: &'static str = Box::leak(format!("diff-line-{index}").into_boxed_str());
        cx.debug_bounds(name)
    });
    let rows: Vec<_> = rows.collect();
    let picked = rows.into_iter().any(|line| {
        click_at(cx, line.center());
        workspace.read_with(cx, |workspace, _| {
            !workspace.shown.viewing.as_ref().unwrap().picked.is_empty()
        })
    });

    assert!(picked, "a click on an added line picks it");
}

#[gpui::test]
fn a_diff_without_lines_says_so(cx: &mut TestAppContext) {
    let (_fixture, workspace, cx) = changed_file(cx);

    update(&workspace, cx, |workspace, cx| {
        workspace.shown.viewing.as_mut().unwrap().lines = Some(Ok(Vec::new()));
        cx.notify();
    });

    assert!(cx.debug_bounds("diff-lines").is_none());
}

#[gpui::test]
fn the_links_of_a_diff_clear_and_stage_the_picked_lines_and_hunks(cx: &mut TestAppContext) {
    let (fixture, workspace, cx) = changed_file(cx);
    let lines = workspace.read_with(cx, |workspace, _| {
        let viewing = workspace.shown.viewing.as_ref().unwrap();
        viewing.lines.clone().unwrap().unwrap()
    });
    assert_eq!(lines[ADDED], "+more");

    click(cx, "diff-line-8");
    assert_eq!(picked(&workspace, cx), [ADDED]);
    click(cx, "diff-line-8");
    assert_eq!(
        picked(&workspace, cx),
        Vec::<usize>::new(),
        "a second click takes the line off"
    );
    click(cx, "diff-line-8");
    click(cx, "clear-lines");
    assert_eq!(picked(&workspace, cx), Vec::<usize>::new());
    click(cx, "diff-line-8");
    click(cx, "apply-lines");
    let staged = git_output(&fixture.alpha, &["show", ":README.md"]);
    assert_eq!(
        staged, "alpha\nmore\n",
        "the kept line gets its newline back"
    );

    git(&fixture.alpha, &["reset", "-q"]);
    update(&workspace, cx, |workspace, cx| {
        workspace.open_file("README.md".into(), Staging::Unstaged, tracked(), cx);
    });
    click(cx, "hunk-4");
    let staged = git_output(&fixture.alpha, &["diff", "--cached"]);
    assert!(staged.contains("+changed"), "{staged}");
}

#[gpui::test]
fn the_end_of_a_text_selection_on_a_line_does_not_pick_it(cx: &mut TestAppContext) {
    let (_fixture, workspace, cx) = changed_file(cx);
    let bounds = cx.debug_bounds("diff-line-8").unwrap();
    let (start, end) = (bounds.origin + point(px(90.), px(4.)), bounds.center());

    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();

    assert_eq!(picked(&workspace, cx), Vec::<usize>::new());
}
