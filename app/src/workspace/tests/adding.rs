//! Adding repositories through the folder picker.

use super::*;

#[gpui::test]
fn the_folder_picker_answers_are_handled_without_waiting_for_it(cx: &mut TestAppContext) {
    use crate::picker::PickOutcome;
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_group(Some("work".into()), cx);
    });
    let root = fixture.alpha.parent().unwrap().to_path_buf();
    let delta = repository(&root, "delta", false);

    // Picked: the folders are added to the selected group.
    workspace.update(cx, |workspace, cx| {
        workspace.progress.picking_folders = true;
        workspace.finish_picking(PickOutcome::Picked(vec![delta.clone()]), cx);
    });
    cx.run_until_parked();
    assert_eq!(names(&workspace, cx), ["alpha", "delta"]);
    workspace.read_with(cx, |workspace, _| {
        assert!(
            !workspace.progress.picking_folders,
            "the waiting notice is gone"
        );
    });

    // Closed by the user: nothing happens and nothing is shown as an error.
    workspace.update(cx, |workspace, cx| {
        workspace.progress.picking_folders = true;
        workspace.finish_picking(PickOutcome::Canceled, cx);
    });
    workspace.read_with(cx, |workspace, _| {
        assert!(!workspace.progress.picking_folders);
        assert!(
            workspace.feedback.notice.is_none(),
            "a cancel is not an error"
        );
    });
    assert_eq!(names(&workspace, cx), ["alpha", "delta"]);

    // Failed: an error shows at once as a red notice and says why.
    workspace.update(cx, |workspace, cx| {
        workspace.progress.picking_folders = true;
        workspace.finish_picking(PickOutcome::Failed("boom".into()), cx);
    });
    workspace.read_with(cx, |workspace, _| {
        assert!(!workspace.progress.picking_folders);
        let notice = workspace.feedback.notice.clone().expect("a visible notice");
        assert!(
            notice.contains("The folder picker failed: boom"),
            "{notice}"
        );
        assert!(
            notice.contains("drop folders"),
            "it points to another way: {notice}"
        );
        let (_, outcome) = workspace.feedback.message.clone().unwrap();
        assert_eq!(outcome, Outcome::Failed);
    });
}

#[gpui::test]
fn adding_repositories_runs_the_picker_away_from_the_window_and_cancel_stops_it(
    cx: &mut TestAppContext,
) {
    use crate::picker::Picker;
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_group(Some("work".into()), cx);
    });

    // A picker that is closed at once: the waiting notice comes and goes.
    workspace.update(cx, |workspace, _| {
        workspace.services.picker = Picker::new("true").marked_by("unused");
    });
    cx.simulate_keystrokes("cmd-o");
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert!(!workspace.progress.picking_folders);
        assert!(
            workspace.feedback.notice.is_none(),
            "a closed picker is not an error"
        );
    });

    // While a picker is open, another is not opened, and Cancel stops the open one.
    let marker = format!("naran-workspace-picker-{}", std::process::id());
    let mut open_picker = Command::new("sh")
        .args(["-c", "sleep 30; true", &marker])
        .spawn()
        .unwrap();
    workspace.update(cx, |workspace, cx| {
        workspace.services.picker = Picker::new("naran-no-such-program").marked_by(&marker);
        workspace.progress.picking_folders = true;
        workspace.prompt_for_repositories(cx);
        cx.notify();
    });
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.feedback.notice.is_none(), "no second picker ran");
    });
    click(cx, "cancel-picking");
    assert_eq!(
        open_picker.wait().unwrap().code(),
        None,
        "the open picker was stopped"
    );
}
