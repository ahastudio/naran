//! Acceptance tests of viewing changes that open files through the file
//! lists and read what the diff pane holds.

use gpui::Modifiers;

use super::*;

fn open_listed(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
    name: &str,
    state: model::FileState,
) {
    workspace.update(cx, |workspace, cx| {
        let file = ListedFile::new(name, Staging::Unstaged, state);
        workspace.click_file(file, Modifiers::default(), cx);
    });
    cx.run_until_parked();
}

fn copy_everything(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Option<String> {
    let area = cx.debug_bounds("diff-lines").expect("the diff is shown");
    cx.simulate_click(
        area.origin + gpui::point(px(80.), px(8.)),
        Modifiers::default(),
    );
    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| workspace.texts.copied())
}

#[gpui::test]
fn a_modified_file_shows_its_lines_without_the_git_header_or_markers(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("one.txt"), "keep\ngone\n").unwrap();
    git(&fixture.alpha, &["add", "one.txt"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "one"]);
    std::fs::write(fixture.alpha.join("one.txt"), "keep\nnew\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    open_listed(&workspace, cx, "one.txt", state('.', 'M', false));

    let copied = copy_everything(&workspace, cx).expect("the text is selected");

    assert_eq!(copied, "keep\ngone\nnew");
}

#[gpui::test]
fn an_untracked_file_that_is_big_or_not_text_is_told_so_in_the_pane(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("big.txt"), vec![b'x'; 600 * 1024]).unwrap();
    std::fs::write(fixture.alpha.join("raw.bin"), [0xff, 0xfe, 0x00]).unwrap();
    let (workspace, cx) = open(&fixture, cx);
    let untracked = state('?', '?', false);

    open_listed(&workspace, cx, "big.txt", untracked);
    let big = diff(&workspace, cx).unwrap();
    assert!(
        big[0].starts_with("The file is too large to show"),
        "{big:?}"
    );
    open_listed(&workspace, cx, "raw.bin", untracked);
    assert_eq!(diff(&workspace, cx).unwrap(), ["The file is not text."]);
}

#[gpui::test]
fn a_long_file_is_cut_and_a_note_says_how_much_is_left(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("long.txt"), "x\n".repeat(3005)).unwrap();
    let (workspace, cx) = open(&fixture, cx);

    open_listed(&workspace, cx, "long.txt", state('?', '?', false));

    assert!(cx.debug_bounds("omitted-note").is_some());
}

#[gpui::test]
fn a_picture_over_the_size_limit_is_refused_with_an_error(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(
        fixture.alpha.join("huge.png"),
        vec![0u8; 8 * 1024 * 1024 + 1],
    )
    .unwrap();
    let (workspace, cx) = open(&fixture, cx);

    open_listed(&workspace, cx, "huge.png", state('?', '?', false));

    workspace.read_with(cx, |workspace, _| {
        let viewing = workspace.shown.viewing.as_ref().unwrap();
        let error = viewing.pictures.as_ref().unwrap().as_ref().err().unwrap();
        assert!(error.to_string().contains("huge.png"), "{error}");
    });
}

#[gpui::test]
fn a_staged_image_shows_the_committed_picture_and_the_staged_one(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("pic.png"), TINY_PNG).unwrap();
    git(&fixture.alpha, &["add", "pic.png"]);
    git(&fixture.alpha, &["commit", "-q", "-m", "picture"]);
    let mut changed = TINY_PNG.to_vec();
    changed.extend([0, 0]);
    std::fs::write(fixture.alpha.join("pic.png"), &changed).unwrap();
    git(&fixture.alpha, &["add", "pic.png"]);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        let file = ListedFile::new("pic.png", Staging::Staged, state('M', '.', false));
        workspace.click_file(file, Modifiers::default(), cx);
    });
    cx.run_until_parked();

    assert_eq!(pictures(&workspace, cx), (true, true));
}

#[gpui::test]
fn select_all_in_a_list_of_one_file_opens_it_without_a_stack(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("one.txt"), "1\n").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, _| workspace.focus_pane(Pane::Files));

    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();

    assert!(stack(&workspace, cx).is_none());
    assert_eq!(diff(&workspace, cx), Some(vec!["+1".to_string()]));
}

#[gpui::test]
fn the_stack_of_a_commit_has_its_lines_and_a_note_for_its_image(cx: &mut TestAppContext) {
    use super::detail::StackRow;
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("one.txt"), "one\n").unwrap();
    std::fs::write(fixture.alpha.join("pic.png"), TINY_PNG).unwrap();
    git(&fixture.alpha, &["add", "."]);
    git(&fixture.alpha, &["commit", "-q", "-m", "two files"]);
    let (workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();
    workspace.update(cx, |workspace, _| workspace.focus_pane(Pane::Files));
    cx.simulate_keystrokes("cmd-a");
    cx.run_until_parked();

    let rows = stack(&workspace, cx).expect("both files are stacked");

    assert!(rows.contains(&StackRow::Line("+one".into())));
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(row, StackRow::Note(_)))
            .count(),
        1,
        "the image has a note"
    );
}
