//! The stacked diffs of two or more picked files: what is read for them and
//! how the rows are laid out.

use super::*;
use crate::workspace::detail::stack::{
    IMAGE_NOTE, StackJob, StackRow, read_stack, stack_file_rows, stack_job,
};
use crate::workspace::detail::stack_view::{StackScroll, columns, sticky_head, widest};
use crate::workspace::detail::versions::Compared;

/// The stacked diffs scrolled down by `scrolled` pixels, in rows 20 pixels tall.
fn scrolled(scrolled: f32) -> StackScroll {
    StackScroll { scrolled, row: 20. }
}

#[test]
fn letters_outside_ascii_take_two_columns() {
    assert_eq!(columns("abc"), 3);
    assert_eq!(columns("한글 a"), 6);
    assert_eq!(columns(""), 0);
}

#[test]
fn the_widest_stacked_row_is_found_by_its_columns() {
    let rows = vec![
        StackRow::File {
            path: "a.txt".into(),
            badge: '?',
        },
        StackRow::Line("+short".into()),
        StackRow::Line("+\t\tx".into()), // tabs are drawn as four spaces each
        StackRow::Note("a note".into()),
    ];
    assert_eq!(widest(&rows), Some(2));
    let korean = vec![
        StackRow::Line("+abcdefgh".into()),
        StackRow::Line("+한글한글한".into()),
    ];
    assert_eq!(
        widest(&korean),
        Some(1),
        "five Korean letters are wider than eight ASCII ones"
    );
    let named = vec![
        StackRow::Line("+x".into()),
        StackRow::File {
            path: "long/path/name.txt".into(),
            badge: 'M',
        },
    ];
    assert_eq!(widest(&named), Some(1));
    assert_eq!(widest(&[]), None);
}

#[test]
fn the_head_of_the_file_at_the_top_sticks_until_the_next_head_pushes_it() {
    let head = |path: &str| StackRow::File {
        path: path.into(),
        badge: 'M',
    };
    let line = || StackRow::Line("+x".into());
    let rows = vec![head("a"), line(), line(), head("b"), line()];
    assert_eq!(
        sticky_head(&rows, scrolled(0.)),
        None,
        "at the top the first head is in place"
    );
    assert_eq!(sticky_head(&rows, scrolled(25.)), Some((0, 0.)));
    assert_eq!(
        sticky_head(&rows, scrolled(45.)),
        Some((0, -5.)),
        "b comes in and pushes a up"
    );
    assert_eq!(
        sticky_head(&rows, scrolled(60.)),
        Some((3, 0.)),
        "b has fully taken over"
    );
    assert_eq!(
        sticky_head(&rows, scrolled(500.)),
        Some((3, 0.)),
        "past the end stays on the last file"
    );
    assert_eq!(
        sticky_head(&[line(), line()], scrolled(25.)),
        None,
        "no head above"
    );
    assert_eq!(sticky_head(&[], scrolled(25.)), None);
}

#[test]
fn only_a_file_head_has_a_name_and_a_badge() {
    let head = StackRow::File {
        path: "a.txt".into(),
        badge: 'M',
    };
    assert_eq!(head.file(), Some(("a.txt", 'M')));
    assert_eq!(StackRow::Note("a note".into()).file(), None);
    assert_eq!(StackRow::Line("+x".into()).file(), None);
}

#[test]
fn an_image_among_the_picked_files_gets_a_note_instead_of_lines() {
    let job = StackJob {
        path: "pic.png".into(),
        compared: Compared::Untracked,
        badge: '?',
    };

    let rows = stack_file_rows(Path::new("/unused"), &job).unwrap();

    assert_eq!(
        rows,
        [
            StackRow::File {
                path: "pic.png".into(),
                badge: '?'
            },
            StackRow::Note(IMAGE_NOTE.into())
        ]
    );
}

#[test]
fn a_file_that_is_no_longer_changed_is_left_out_of_the_stack() {
    assert!(stack_job(&[], "README.md", Staging::Unstaged).is_none());
}

#[test]
fn a_stack_that_cannot_be_read_is_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let job = StackJob {
        path: "README.md".into(),
        compared: Compared::Unstaged,
        badge: 'M',
    };

    assert!(read_stack(directory.path(), &[job], 0).is_err());
}

#[gpui::test]
fn the_same_picked_files_are_not_read_again(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    for name in ["a.txt", "b.txt"] {
        std::fs::write(fixture.alpha.join(name), name).unwrap();
    }
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.select_all_files(Staging::Unstaged, cx);
        assert!(workspace.shown.stacked.is_some());
        workspace.refresh_stacked(cx);
        assert!(workspace.shown.stacked.is_some());
    });
}

#[gpui::test]
fn picking_the_same_files_twice_and_a_single_file_list_are_handled(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("one.txt"), "1").unwrap();
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.select_all_files(Staging::Unstaged, cx);
        assert_eq!(workspace.shown.viewing.as_ref().unwrap().file, "one.txt");
        workspace.step_file(1, cx);
        workspace.shown.viewing = Some(viewing(&fixture.alpha, "ghost.txt"));
        workspace.extend_files(1, cx);
        workspace.shown.marked.paths = ["one.txt".to_string(), "ghost.txt".to_string()].into();
        workspace.refresh_stacked(cx);
    });
}
