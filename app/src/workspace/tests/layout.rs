//! Dividers and the saved layout.

use super::*;

#[gpui::test]
fn dragging_a_divider_resizes_the_part_next_to_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.interaction.dragging = Some(Divider::Groups);
        workspace.drag(gpui::point(px(260.), px(100.)), cx);
        assert_eq!(workspace.layout.sizes.groups, 260.);
        workspace.interaction.dragging = Some(Divider::Commits);
        workspace.drag(gpui::point(px(0.), px(344.)), cx);
        assert_eq!(workspace.layout.sizes.commits, 300.);
        workspace.drag(gpui::point(px(0.), px(5.)), cx);
        assert_eq!(
            workspace.layout.sizes.commits, 80.,
            "never smaller than the minimum"
        );
    });
}

#[gpui::test]
fn the_layout_is_saved_as_it_changes(cx: &mut TestAppContext) {
    use super::SavedLayout;
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let file = fixture.config.join("layout.json");

    workspace.update(cx, |workspace, cx| {
        workspace.interaction.dragging = Some(Divider::Groups);
        workspace.drag(gpui::point(px(260.), px(100.)), cx);
        workspace.interaction.dragging = Some(Divider::Commits);
        workspace.drag(gpui::point(px(0.), px(344.)), cx);
        workspace.interaction.dragging = Some(Divider::Files);
        workspace.drag(gpui::point(px(900.), px(0.)), cx);
        workspace.focus_pane(Pane::Commits);
    });
    let saved: SavedLayout = store::load(&file);
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(
            saved.sizes, workspace.layout.sizes,
            "what is on disk is what is on screen"
        );
    });
    assert_eq!(saved.sizes.groups, 260.);
    assert_eq!(saved.sizes.commits, 300.);
    assert_eq!(saved.pane, Pane::Commits);
}

#[test]
fn a_broken_layout_file_cannot_hide_a_part() {
    use super::SavedLayout;
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("layout.json");

    // Missing file and unreadable file: the defaults.
    assert_eq!(store::load::<SavedLayout>(&file), SavedLayout::default());
    std::fs::write(&file, "not json").unwrap();
    assert_eq!(store::load::<SavedLayout>(&file), SavedLayout::default());

    // A part missing from the file keeps its default; wild numbers are pulled in range.
    std::fs::write(
        &file,
        r#"{"sizes":{"groups":5.0,"commits":99999.0},"pane":"Files"}"#,
    )
    .unwrap();
    let layout: SavedLayout = store::load(&file);
    assert_eq!(layout.pane, Pane::Files);
    let sizes = layout.sizes.clamped();
    assert_eq!(sizes.groups, 120.);
    assert_eq!(sizes.commits, 700.);
    assert_eq!(sizes.repositories, Sizes::DEFAULT.repositories);
    assert_eq!(
        Sizes {
            groups: f32::NAN,
            ..Sizes::DEFAULT
        }
        .clamped()
        .groups,
        Sizes::DEFAULT.groups
    );
}

#[gpui::test]
fn the_column_heads_of_the_commit_list_stay_on_one_line_when_it_is_narrow(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let wide = cx
        .debug_bounds("history-head")
        .expect("the heads are shown");
    // Wide side columns leave the commit list only a little room.
    workspace.update(cx, |workspace, cx| {
        workspace.layout.sizes.groups = 700.;
        workspace.layout.sizes.repositories = 800.;
        cx.notify();
    });
    cx.run_until_parked();
    let narrow = cx.debug_bounds("history-head").unwrap();
    assert!(
        narrow.size.width < px(450.),
        "the list is narrow: {narrow:?}"
    );
    assert_eq!(narrow.size.height, wide.size.height, "still one line");
}

#[gpui::test]
fn the_divider_next_to_the_repositories_moves_their_column(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.interaction.dragging = Some(Divider::Repositories);
        let edge = workspace.layout.sizes.groups + 300. + DIVIDER_WIDTH;
        workspace.drag(gpui::point(px(edge), px(0.)), cx);
        assert_eq!(workspace.layout.sizes.repositories, 300.);
    });
}

#[gpui::test]
fn a_notice_is_dismissed_and_a_divider_is_grabbed_and_let_go(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.feedback.notice = Some("a notice".into());
        workspace.dismiss_notice(cx);
        assert!(workspace.feedback.notice.is_none());
        workspace.start_drag(Divider::Groups, cx);
        assert_eq!(workspace.interaction.dragging, Some(Divider::Groups));
        workspace.release_drag(cx);
        assert_eq!(workspace.interaction.dragging, None);
        workspace.release_drag(cx);
    });
}
