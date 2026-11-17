//! The Staged and Unstaged lists: picking files with clicks and keys, and
//! dragging them from one list to the other.

use gpui::{MouseButton, point, px};

use super::*;
use crate::workspace::detail::file_rows::{DraggedFile, drag_preview};
use crate::workspace::detail::marks::Marked;

#[gpui::test]
fn picks_without_an_anchor_start_from_the_cursor(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let files = [
        ("a.txt".to_owned(), tracked()),
        ("b.txt".to_owned(), tracked()),
    ];

    update(&workspace, cx, |workspace, _| {
        workspace.shown.marked = Marked::default();
        workspace.mark_range("a.txt", Staging::Unstaged);
        assert!(
            workspace.shown.marked.paths.is_empty(),
            "no anchor, no range"
        );

        let cursor = ("a.txt".into(), Staging::Unstaged);
        let next = workspace.mark_extension(&files, &cursor, 1);

        assert_eq!(next, Some(1));
        assert_eq!(workspace.shown.marked.anchor.as_deref(), Some("a.txt"));
    });
}

#[gpui::test]
fn picks_of_files_that_left_the_list_are_dropped(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    update(&workspace, cx, |workspace, _| {
        workspace.shown.marked.paths.insert("gone.txt".into());
        workspace.drop_unlisted_marks();
        assert!(workspace.shown.marked.paths.is_empty());
    });
}

#[gpui::test]
fn dragging_several_files_shows_how_many_and_the_picks_can_be_cleared(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let several = DraggedFile {
        paths: vec!["a".into(), "b".into()],
        from: Staging::Unstaged,
    };

    let preview = cx.update(|window, cx| drag_preview(&several, point(px(0.), px(0.)), window, cx));

    assert_eq!(
        preview.read_with(cx, |preview, _| preview.name.clone()),
        "2 files"
    );
    workspace.update(cx, |workspace, cx| {
        let mut open = viewing(&fixture.alpha, "a.txt");
        open.picked = [1, 2].into();
        workspace.shown.viewing = Some(open);
        workspace.clear_picks(cx);
        assert!(workspace.shown.viewing.as_ref().unwrap().picked.is_empty());
    });
}

#[gpui::test]
fn clicking_a_file_opens_it_and_dragging_it_to_the_other_list_stages_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    select(&workspace, &fixture.beta, cx);
    let row = cx
        .debug_bounds("file-unstaged-draft.txt")
        .expect("the unstaged file is listed");

    click_at(cx, row.center());
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.shown.viewing.as_ref().unwrap().file, "draft.txt");
    });

    let target = cx.debug_bounds("list-staged").unwrap().center();
    cx.simulate_mouse_down(row.center(), MouseButton::Left, Modifiers::none());
    for step in 1..=8_u8 {
        let at = row.center() + (target - row.center()) * (f32::from(step) / 8.);
        cx.simulate_mouse_move(at, MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
    }
    cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();

    assert!(
        cx.debug_bounds("file-staged-draft.txt").is_some(),
        "the file moved to Staged files"
    );
}
