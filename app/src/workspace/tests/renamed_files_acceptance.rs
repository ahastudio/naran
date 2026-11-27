//! How a renamed file shows in the Staged list, driven through the window.

use super::staging_acceptance::{click_row, listed};
use super::*;

#[gpui::test]
fn a_renamed_file_shows_its_new_path_then_the_old_one_and_the_others_show_their_path(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    git_output(&fixture.alpha, &["mv", "README.md", "GUIDE.md"]);
    std::fs::write(fixture.alpha.join("loose.txt"), "one\n").unwrap();
    let (_workspace, cx) = open(&fixture, cx);

    assert!(listed(cx, "label-staged-GUIDE.md <- README.md"));
    assert!(listed(cx, "label-unstaged-loose.txt"));
    assert!(!listed(cx, "label-staged-GUIDE.md"));
}

#[gpui::test]
fn a_renamed_file_shows_only_the_lines_that_changed(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    commit(&fixture.alpha, "notes.txt", "one\ntwo\nthree\n", "notes");
    git_output(&fixture.alpha, &["mv", "notes.txt", "guide.txt"]);
    std::fs::write(fixture.alpha.join("guide.txt"), "one\nTWO\nthree\n").unwrap();
    git_output(&fixture.alpha, &["add", "guide.txt"]);
    let (workspace, cx) = open(&fixture, cx);

    click_row(cx, "file-staged-guide.txt", gpui::Modifiers::none());

    let lines = diff(&workspace, cx).expect("the diff is open");
    assert!(lines.contains(&"+TWO".to_string()));
    assert!(lines.contains(&"-two".to_string()));
    assert!(!lines.contains(&"+one".to_string()), "{lines:?}");
}

#[gpui::test]
fn a_rename_without_edits_opens_a_diff_that_says_what_the_file_was(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    commit(&fixture.alpha, "notes.txt", "one\ntwo\n", "notes");
    git_output(&fixture.alpha, &["mv", "notes.txt", "guide.txt"]);
    let (workspace, cx) = open(&fixture, cx);

    click_row(cx, "file-staged-guide.txt", gpui::Modifiers::none());

    let lines = diff(&workspace, cx).expect("the diff is open");
    assert!(
        lines.contains(&"rename from notes.txt".to_string()),
        "{lines:?}"
    );
    assert!(!lines.iter().any(|line| line.starts_with('@')));
}
