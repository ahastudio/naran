//! The repository list: adding, ordering and selecting repositories.

use super::*;

#[gpui::test]
fn adds_folders_and_reports_the_ones_that_are_not_repositories(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_group(Some("work".into()), cx);
    });
    let root = fixture.alpha.parent().unwrap().to_path_buf();
    let delta = repository(&root, "delta", false);
    let plain = root.join("plain");
    std::fs::create_dir_all(&plain).unwrap();

    workspace.update(cx, |workspace, cx| {
        workspace.add_repositories(vec![delta.clone(), plain.clone()], cx);
    });
    cx.run_until_parked();

    assert_eq!(names(&workspace, cx), ["alpha", "delta"]);
    assert_eq!(selected(&workspace, cx), Some(delta.clone()));
    let paths: Vec<PathBuf> = store::load(&fixture.config.join("repositories.json"));
    assert_eq!(paths.last(), Some(&delta));
    assert!(!paths.contains(&plain));
    workspace.read_with(cx, |workspace, _| {
        let (message, outcome) = workspace.feedback.message.clone().unwrap();
        assert_eq!(outcome, Outcome::Failed);
        assert!(message.starts_with("Added 1, failed 1"), "{message}");
        let notice = workspace.feedback.notice.clone().expect("a visible notice");
        assert!(notice.starts_with("Not added:"), "{notice}");
        assert!(notice.contains("plain"), "{notice}");
    });
}

#[gpui::test]
fn the_status_bar_shows_the_path_of_the_selected_repository(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(
            workspace.selection.repository.as_deref(),
            Some(fixture.alpha.as_path())
        );
    });
}

#[gpui::test]
fn dropping_a_repository_on_another_row_changes_the_order_and_keeps_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1, 2])]);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(names(&workspace, cx), ["alpha", "beta", "gamma"]);

    workspace.update(cx, |workspace, cx| {
        workspace.move_repository(&fixture.gamma, &fixture.alpha, cx);
    });
    assert_eq!(names(&workspace, cx), ["gamma", "alpha", "beta"]);
    let settings: Settings = store::load(&fixture.config.join("settings.json"));
    let saved: Vec<PathBuf> = settings.group("work").unwrap().paths.clone();
    assert_eq!(
        saved,
        [
            fixture.gamma.clone(),
            fixture.alpha.clone(),
            fixture.beta.clone()
        ]
    );

    // Dropped on itself: nothing changes.
    workspace.update(cx, |workspace, cx| {
        workspace.move_repository(&fixture.alpha, &fixture.alpha, cx);
    });
    assert_eq!(names(&workspace, cx), ["gamma", "alpha", "beta"]);
}

#[gpui::test]
fn dragging_a_repository_row_with_the_mouse_reorders_the_group(cx: &mut TestAppContext) {
    use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, point, px};
    let fixture = fixture(&[("work", &[0, 1, 2])]);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(names(&workspace, cx), ["alpha", "beta", "gamma"]);

    // Rows are about 61px tall, starting below the toolbar and the header.
    let (x, first, third) = (400., 44. + 40. + 30., 44. + 40. + 61. * 2. + 30.);
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position: point(px(x), px(third)),
        modifiers: Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    for y in [third - 10., third - 30., third - 60., first + 10., first] {
        cx.simulate_event(MouseMoveEvent {
            position: point(px(x), px(y)),
            pressed_button: Some(MouseButton::Left),
            modifiers: Modifiers::default(),
        });
        cx.run_until_parked();
    }
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: point(px(x), px(first)),
        modifiers: Modifiers::default(),
        click_count: 1,
    });
    cx.run_until_parked();
    assert_eq!(names(&workspace, cx), ["gamma", "alpha", "beta"]);
}

#[gpui::test]
fn without_a_selected_group_moving_removing_and_editing_do_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, _| workspace.selection.group = None);

    workspace.update(cx, |workspace, cx| {
        workspace.move_repository(&fixture.alpha, &fixture.beta, cx);
        workspace.remove(&fixture.alpha, cx);
    });
    cx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.confirm_remove(fixture.alpha.clone(), window, cx);
            workspace.finish_editing("anything", window, cx);
        });
    });

    assert!(!cx.has_pending_prompt());
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.settings.groups[0].paths.len(), 2);
        assert!(workspace.interaction.editing.is_none());
    });
}

#[gpui::test]
fn clicking_a_repository_row_selects_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1, 2])]);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));

    // Rows are about 61px tall, starting below the toolbar and the header.
    let second = gpui::point(px(400.), px(44. + 40. + 61. + 30.));
    cx.simulate_click(second, gpui::Modifiers::default());
    cx.run_until_parked();

    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));
}

#[gpui::test]
fn files_dropped_on_the_window_are_added_to_the_group(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    let position = gpui::point(px(400.), px(300.));

    cx.simulate_event(gpui::FileDropEvent::Entered {
        position,
        paths: ExternalPaths::default(),
    });
    cx.simulate_event(gpui::FileDropEvent::Submit { position });
    cx.run_until_parked();

    workspace.read_with(cx, |workspace, _| {
        let message = workspace.feedback.message.clone().unwrap().0;
        assert!(message.contains("Added 0"), "{message}");
    });
}
