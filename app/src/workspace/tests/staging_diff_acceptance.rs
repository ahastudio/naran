//! Staging parts of a file, whole files that cannot be split, and what the
//! lists show, driven through the window.

use gpui::{Modifiers, MouseButton};

use super::staging_acceptance::{center, click_row, conflict, drag, listed};
use super::*;

#[gpui::test]
fn untracked_and_conflicted_files_have_no_partial_staging_but_stage_whole(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    conflict(&fixture);
    std::fs::write(fixture.alpha.join("loose.txt"), "one\ntwo\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);

    for file in ["loose.txt", "README.md"] {
        click_row(cx, &format!("file-unstaged-{file}"), Modifiers::none());
        assert!(!listed(cx, "hunk-0"), "{file} has no hunk link");
        if listed(cx, "diff-line-0") {
            click(cx, "diff-line-0");
        }
        assert!(!listed(cx, "apply-lines"), "{file} has no line link");
        let none = workspace.read_with(cx, |workspace, _| {
            workspace.shown.viewing.as_ref().unwrap().picked.is_empty()
        });
        assert!(none, "{file} has no pickable lines");
    }

    drag(cx, "file-unstaged-README.md", "list-staged");
    assert_eq!(git_output(&fixture.alpha, &["ls-files", "--unmerged"]), "");
    drag(cx, "file-unstaged-loose.txt", "list-staged");
    assert!(listed(cx, "file-staged-loose.txt"));
}

#[gpui::test]
fn every_file_of_a_new_folder_is_listed_and_a_conflict_is_only_in_unstaged(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    conflict(&fixture);
    std::fs::create_dir(fixture.alpha.join("dir")).unwrap();
    for name in ["x.txt", "y.txt", "z.txt"] {
        std::fs::write(fixture.alpha.join("dir").join(name), name).unwrap();
    }
    let (_workspace, cx) = open(&fixture, cx);

    for name in ["x.txt", "y.txt", "z.txt"] {
        assert!(listed(cx, &format!("file-unstaged-dir/{name}")), "{name}");
    }
    assert!(listed(cx, "file-unstaged-README.md"));
    assert!(!listed(cx, "file-staged-README.md"));
}

#[gpui::test]
fn a_failed_drop_shows_a_notice_and_the_next_one_clears_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    four_untracked_files(&fixture);
    let (workspace, cx) = open(&fixture, cx);
    let row = center(cx, "file-unstaged-a.txt");
    std::fs::remove_file(fixture.alpha.join("a.txt")).unwrap();
    let to = center(cx, "list-staged");
    cx.simulate_mouse_down(row, MouseButton::Left, Modifiers::none());
    for step in 1..=8_u8 {
        cx.simulate_mouse_move(
            row + (to - row) * (f32::from(step) / 8.),
            MouseButton::Left,
            Modifiers::none(),
        );
    }
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let notice = workspace.read_with(cx, |workspace, _| workspace.feedback.notice.clone());
    assert!(notice.is_some_and(|text| text.starts_with("Stage failed:")));

    drag(cx, "file-unstaged-b.txt", "list-staged");
    let cleared = workspace.read_with(cx, |workspace, _| workspace.feedback.notice.is_none());
    assert!(cleared);
}

#[gpui::test]
fn a_hunk_is_staged_and_unstaged_from_the_diff_links(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("README.md"), "alpha\nsecond\n").unwrap();
    let (_workspace, cx) = open(&fixture, cx);
    click_row(cx, "file-unstaged-README.md", Modifiers::none());
    click(cx, "hunk-4");
    assert!(git_output(&fixture.alpha, &["diff", "--cached"]).contains("+second"));

    click_row(cx, "file-staged-README.md", Modifiers::none());
    click(cx, "hunk-4");
    assert_eq!(git_output(&fixture.alpha, &["diff", "--cached"]), "");
}

#[gpui::test]
fn picked_lines_stage_and_unstage_from_the_diff_links(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("README.md"), "alpha\nsecond\nthird\n").unwrap();
    let (_workspace, cx) = open(&fixture, cx);
    click_row(cx, "file-unstaged-README.md", Modifiers::none());
    click(cx, "diff-line-8");
    click(cx, "apply-lines");
    let staged = git_output(&fixture.alpha, &["diff", "--cached"]);
    assert!(
        staged.contains("+second") && !staged.contains("+third"),
        "{staged}"
    );

    click_row(cx, "file-staged-README.md", Modifiers::none());
    click(cx, "diff-line-8");
    click(cx, "apply-lines");
    let staged = git_output(&fixture.alpha, &["diff", "--cached"]);
    assert!(!staged.contains("+second"), "{staged}");
}

#[test]
fn badges_name_the_change_and_a_conflict_is_always_marked() {
    let change = |index, worktree, conflicted| Change {
        path: "f".into(),
        original: None,
        index,
        worktree,
        conflicted,
    };
    let badge = |change: &Change, side| model::change_badge(change, side);
    assert_eq!(badge(&change('A', '.', false), Staging::Staged), 'A');
    assert_eq!(badge(&change('D', '.', false), Staging::Staged), 'D');
    assert_eq!(badge(&change('R', '.', false), Staging::Staged), 'R');
    assert_eq!(badge(&change('.', 'M', false), Staging::Unstaged), 'M');
    assert_eq!(badge(&change('?', '?', false), Staging::Unstaged), '?');
    assert_eq!(badge(&change('U', 'U', true), Staging::Unstaged), '!');
    assert_eq!(badge(&change('M', 'M', false), Staging::Staged), 'M');
}

#[gpui::test]
fn a_line_toggles_clear_drops_the_picks_and_no_link_shows_without_picks(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("README.md"), "alpha\nsecond\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    click_row(cx, "file-unstaged-README.md", Modifiers::none());
    assert!(!listed(cx, "apply-lines"), "no picks, no Stage lines link");

    click(cx, "diff-line-8");
    assert!(listed(cx, "apply-lines"));
    click(cx, "diff-line-8");
    let none = workspace.read_with(cx, |workspace, _| {
        workspace.shown.viewing.as_ref().unwrap().picked.is_empty()
    });
    assert!(none, "a second click takes the line off");

    click(cx, "diff-line-8");
    click(cx, "clear-lines");
    let none = workspace.read_with(cx, |workspace, _| {
        workspace.shown.viewing.as_ref().unwrap().picked.is_empty()
    });
    assert!(none, "Clear drops the picks");
}

#[gpui::test]
fn a_line_added_after_an_unterminated_last_line_stages_with_its_newline(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("README.md"), "alpha\nmore\n").unwrap();
    let (_workspace, cx) = open(&fixture, cx);
    click_row(cx, "file-unstaged-README.md", Modifiers::none());

    click(cx, "diff-line-8");
    click(cx, "apply-lines");

    let staged = git_output(&fixture.alpha, &["show", ":README.md"]);
    assert_eq!(staged, "alpha\nmore\n");
}

#[gpui::test]
fn a_file_with_staged_and_unstaged_changes_is_in_both_lists(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("README.md"), "alpha\none\n").unwrap();
    git(&fixture.alpha, &["add", "README.md"]);
    std::fs::write(fixture.alpha.join("README.md"), "alpha\none\ntwo\n").unwrap();
    let (_workspace, cx) = open(&fixture, cx);

    assert!(listed(cx, "file-staged-README.md"));
    assert!(listed(cx, "file-unstaged-README.md"));
}
