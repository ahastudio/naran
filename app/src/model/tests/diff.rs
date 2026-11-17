//! Diff lines, hunks and the files of the staged and working directory lists.

use std::ops::Range;

use naran_core::{Change, Staging};

use super::*;

fn change(index: char, worktree: char, conflicted: bool) -> Change {
    Change {
        path: "a".into(),
        original: None,
        index,
        worktree,
        conflicted,
    }
}

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

#[test]
fn an_open_diff_stays_on_its_side_or_follows_the_file() {
    let both = FileState {
        kind: FileKind::Tracked,
        staged: true,
        unstaged: true,
    };
    let only_staged = FileState {
        unstaged: false,
        ..both
    };
    let only_unstaged = FileState {
        staged: false,
        ..both
    };
    let neither = FileState {
        staged: false,
        unstaged: false,
        ..both
    };
    let (staged, unstaged) = (Staging::Staged, Staging::Unstaged);
    assert_eq!(both.open_side(staged), Some(staged));
    assert_eq!(both.open_side(unstaged), Some(unstaged));
    assert_eq!(only_staged.open_side(unstaged), Some(staged));
    assert_eq!(only_unstaged.open_side(staged), Some(unstaged));
    assert_eq!(neither.open_side(staged), None);
}

#[test]
fn sorts_files_into_the_staged_and_working_directory_lists() {
    let state = |index, worktree| FileState::of(&change(index, worktree, false));

    assert_eq!(
        state('?', '?'),
        FileState {
            kind: FileKind::Untracked,
            staged: false,
            unstaged: true
        }
    );
    assert_eq!(
        state('M', '.'),
        FileState {
            kind: FileKind::Tracked,
            staged: true,
            unstaged: false
        }
    );
    assert_eq!(
        state('.', 'M'),
        FileState {
            kind: FileKind::Tracked,
            staged: false,
            unstaged: true
        }
    );
    assert_eq!(
        state('A', 'M'),
        FileState {
            kind: FileKind::Tracked,
            staged: true,
            unstaged: true
        }
    );
    assert_eq!(
        FileState::of(&change('A', 'A', true)),
        FileState {
            kind: FileKind::Conflicted,
            staged: false,
            unstaged: true
        }
    );
}

#[test]
fn classifies_diff_lines() {
    assert_eq!(classify("+++ b/a.txt"), LineKind::Meta);
    assert_eq!(classify("--- a/a.txt"), LineKind::Meta);
    assert_eq!(classify("diff --git a/a b/a"), LineKind::Meta);
    assert_eq!(classify("\\ No newline at end of file"), LineKind::Meta);
    assert_eq!(classify("@@ -1 +1,2 @@"), LineKind::Hunk);
    assert_eq!(classify("+added"), LineKind::Added);
    assert_eq!(classify("-removed"), LineKind::Removed);
    assert_eq!(classify(" same"), LineKind::Context);
    assert_eq!(untracked_as_diff("a\nb"), ["+a", "+b"]);
}

#[test]
fn finds_hunks_and_their_changed_lines() {
    let diff = lines(
        "diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -1,2 +1,2 @@\n-one\n+uno\n two\n@@ -9 +9 @@\n-nine\n+nueve",
    );

    let found = hunks(&diff);

    assert_eq!(found, [3..7, 7..10]);
    assert_eq!(changed_lines(&diff, found[0].clone()), [4, 5]);
    assert_eq!(changed_lines(&diff, found[1].clone()), [8, 9]);
    assert_eq!(hunks(&lines("diff --git a/a b/a")), [] as [Range<usize>; 0]);
}

#[test]
fn diff_rows_number_lines_and_drop_the_file_header() {
    let lines: Vec<String> = [
        "diff --git a/f b/f",
        "index 1..2 100644",
        "--- a/f",
        "+++ b/f",
        "@@ -3,3 +3,4 @@ fn main",
        " keep",
        "-gone",
        "+new one",
        "+new two",
        "\\ No newline at end of file",
        " tail",
        "@@ -20 +21,0 @@",
        "-x",
    ]
    .map(String::from)
    .to_vec();
    let rows = diff_rows(&lines);
    let line = |index, kind, old, new, text: &str| DiffRow::Line {
        index,
        kind,
        old,
        new,
        text: text.into(),
    };
    assert_eq!(
        rows,
        vec![
            DiffRow::Hunk {
                index: 4,
                title: "Hunk 1 : Lines 3-6".into()
            },
            line(5, LineKind::Context, Some(3), Some(3), "keep"),
            line(6, LineKind::Removed, Some(4), None, "gone"),
            line(7, LineKind::Added, None, Some(4), "new one"),
            line(8, LineKind::Added, None, Some(5), "new two"),
            DiffRow::Note {
                index: 9,
                text: "No newline at end of file".into()
            },
            line(10, LineKind::Context, Some(5), Some(6), "tail"),
            DiffRow::Hunk {
                index: 11,
                title: "Hunk 2 : Line 20".into()
            },
            line(12, LineKind::Removed, Some(20), None, "x"),
        ]
    );
}

#[test]
fn diff_rows_show_an_untracked_file_as_added_lines_and_keep_messages() {
    let rows = diff_rows(&untracked_as_diff("a\n\tb\n"));
    assert_eq!(
        rows,
        vec![
            DiffRow::Line {
                index: 0,
                kind: LineKind::Added,
                old: None,
                new: Some(1),
                text: "a".into()
            },
            DiffRow::Line {
                index: 1,
                kind: LineKind::Added,
                old: None,
                new: Some(2),
                text: "    b".into()
            },
        ]
    );
    let rows = diff_rows(&["The file is not text.".to_string()]);
    assert_eq!(
        rows,
        vec![DiffRow::Note {
            index: 0,
            text: "The file is not text.".into()
        }]
    );
}

#[test]
fn a_context_line_without_its_leading_space_is_kept_as_it_is() {
    let rows = diff_rows(&lines("@@ -1,2 +1,2 @@\n\tx\n-a\n+b"));
    assert_eq!(
        rows[1],
        DiffRow::Line {
            index: 1,
            kind: LineKind::Context,
            old: Some(1),
            new: Some(1),
            text: "    x".into()
        }
    );
}
