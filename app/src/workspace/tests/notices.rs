//! Notices for failures, and changes made outside the app.

use super::*;

#[gpui::test]
fn shows_changes_made_outside_the_app_without_a_refresh(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let state = |cx: &mut VisualTestContext| {
        workspace.read_with(cx, |workspace, _| {
            let gamma = workspace
                .entries
                .iter()
                .find(|entry| entry.path() == fixture.gamma)
                .unwrap();
            model::describe(gamma).0
        })
    };
    assert_eq!(state(cx), "No upstream");

    std::fs::write(fixture.gamma.join("new.txt"), "edited in a terminal").unwrap();

    for _ in 0..50 {
        cx.run_until_parked();
        if state(cx) == "Changes" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert_eq!(state(cx), "Changes");
}

#[gpui::test]
fn a_failed_stage_shows_a_notice_and_the_next_success_clears_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(fixture.beta.clone()), cx);
    });
    cx.run_until_parked();

    workspace.update(cx, |workspace, cx| {
        workspace.stage(vec!["missing-file.txt".into()], cx);
    });
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        let notice = workspace.feedback.notice.clone().expect("a visible notice");
        assert!(notice.starts_with("Stage failed:"), "{notice}");
    });

    workspace.update(cx, |workspace, cx| {
        workspace.stage(vec!["draft.txt".into()], cx);
    });
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.feedback.notice.is_none());
    });
}

#[gpui::test]
fn every_error_shows_as_a_notice_at_once(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    // A settings file that cannot be written: its folder is a plain file.
    let blocker = fixture.config.join("repositories.json");
    workspace.update(cx, |workspace, _| {
        workspace.files.settings = blocker.join("nope.json");
    });
    workspace.update(cx, |workspace, _| workspace.save_settings());
    workspace.read_with(cx, |workspace, _| {
        let notice = workspace
            .feedback
            .notice
            .clone()
            .expect("the failure is shown at once");
        assert!(notice.contains("Could not save the settings"), "{notice}");
    });

    // A success clears nothing by itself, but the notice can be dismissed and a
    // later error replaces it.
    workspace.update(cx, |workspace, _| workspace.feedback.notice = None);
    workspace.update(cx, |workspace, _| workspace.save_layout());
    workspace.update(cx, |workspace, _| {
        workspace.files.layout = blocker.join("nope.json");
    });
    workspace.update(cx, |workspace, _| workspace.save_layout());
    workspace.read_with(cx, |workspace, _| {
        assert!(
            workspace
                .feedback
                .notice
                .clone()
                .unwrap()
                .contains("Could not save the layout")
        );
    });
}

#[gpui::test]
fn a_selection_or_a_repository_list_that_cannot_be_saved_shows_a_notice(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    let blocker = fixture.config.join("repositories.json");

    workspace.update(cx, |workspace, _| {
        workspace.files.selection = blocker.join("nope.json");
        workspace.save_selection();
    });
    workspace.read_with(cx, |workspace, _| {
        let message = workspace.feedback.message.clone().unwrap().0;
        assert!(
            message.contains("Could not save the selection"),
            "{message}"
        );
    });
    workspace.update(cx, |workspace, _| {
        workspace.files.repositories = blocker.join("nope.json");
        workspace.save_paths();
    });
    workspace.read_with(cx, |workspace, _| {
        let message = workspace.feedback.message.clone().unwrap().0;
        assert!(
            message.contains("Could not save the repository list"),
            "{message}"
        );
    });
}

#[gpui::test]
fn watching_that_cannot_start_is_reported_and_leaves_nothing_watched(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, _| {
        let watching = workspace.watch_all(Err(notify::Error::generic("no watcher")));
        assert!(watching.is_none());
        let message = workspace.feedback.message.clone().unwrap().0;
        assert!(message.contains("Could not watch"), "{message}");
    });
}

#[gpui::test]
fn an_entry_is_replaced_only_when_it_is_known_and_different(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    let stranger = Entry::Failed {
        path: "/nowhere".into(),
        error: naran_core::Error::InvalidArgument {
            message: "gone".into(),
        },
    };

    workspace.update(cx, |workspace, _| {
        assert!(!workspace.replace_entry(stranger.clone()));
        let same = workspace.entries[0].clone();
        assert!(!workspace.replace_entry(same));
        let other = Entry::Failed {
            path: workspace.entries[0].path().to_path_buf(),
            error: naran_core::Error::InvalidArgument {
                message: "changed".into(),
            },
        };
        assert!(workspace.replace_entry(other.clone()));
        assert_eq!(workspace.entries[0], other);
    });
}
