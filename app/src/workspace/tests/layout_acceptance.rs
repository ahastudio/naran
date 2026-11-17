//! The screen, the dividers, the saved layout and the keys, driven by the
//! mouse and the keyboard the way a person uses them.

use gpui::{Modifiers, MouseMoveEvent, MouseUpEvent, point};

use super::*;

fn press_at(cx: &mut VisualTestContext, position: gpui::Point<Pixels>) {
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position,
        modifiers: Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    cx.run_until_parked();
}

fn move_to(cx: &mut VisualTestContext, x: f32, y: f32) {
    cx.simulate_event(MouseMoveEvent {
        position: point(px(x), px(y)),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::default(),
    });
    cx.run_until_parked();
}

fn let_go(cx: &mut VisualTestContext) {
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: point(px(0.), px(0.)),
        modifiers: Modifiers::default(),
        click_count: 1,
    });
    cx.run_until_parked();
}

fn grab(cx: &mut VisualTestContext, divider: &'static str) {
    let place = cx.debug_bounds(divider).expect(divider).center();
    press_at(cx, place);
}

fn sizes(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Sizes {
    workspace.read_with(cx, |workspace, _| workspace.layout.sizes)
}

fn selected(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Option<PathBuf> {
    workspace.read_with(cx, |workspace, _| workspace.selection.repository.clone())
}

#[gpui::test]
fn the_three_columns_and_their_dividers_are_on_screen(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (_workspace, cx) = open(&fixture, cx);

    let groups = cx.debug_bounds("divider-Groups").unwrap();
    let repositories = cx.debug_bounds("divider-Repositories").unwrap();
    let commits = cx.debug_bounds("divider-Commits").unwrap();
    let files = cx.debug_bounds("divider-Files").unwrap();

    assert!(groups.origin.x < repositories.origin.x);
    assert!(repositories.origin.x < files.origin.x);
    assert!(commits.origin.x >= repositories.origin.x, "in the detail");
}

#[gpui::test]
fn dragging_a_divider_with_the_mouse_resizes_it_and_letting_go_ends_the_drag(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    grab(cx, "divider-Groups");
    move_to(cx, 260., 100.);
    assert_eq!(sizes(&workspace, cx).groups, 260.);
    let_go(cx);
    move_to(cx, 300., 100.);
    assert_eq!(
        sizes(&workspace, cx).groups,
        260.,
        "no drag after letting go"
    );

    grab(cx, "divider-Commits");
    move_to(cx, 100., 344.);
    assert_eq!(sizes(&workspace, cx).commits, 300.);
    let_go(cx);
}

#[gpui::test]
fn a_divider_stops_at_both_ends_of_its_range(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    grab(cx, "divider-Groups");
    move_to(cx, 900., 100.);
    assert_eq!(sizes(&workspace, cx).groups, 400.);
    move_to(cx, 3., 100.);
    assert_eq!(sizes(&workspace, cx).groups, 120.);
    move_to(cx, 150., 100.);
    assert_eq!(sizes(&workspace, cx).groups, 150., "and it comes back");
}

#[gpui::test]
fn a_broken_layout_file_opens_the_window_with_every_part_in_range(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    std::fs::write(
        fixture.config.join("layout.json"),
        r#"{"sizes":{"groups":5.0,"commits":99999.0}}"#,
    )
    .unwrap();
    let (workspace, cx) = open(&fixture, cx);

    let opened = sizes(&workspace, cx);
    assert_eq!(opened.groups, 120.);
    assert_eq!(opened.commits, 700.);
    assert_eq!(opened.repositories, Sizes::DEFAULT.repositories);
    assert!(cx.debug_bounds("divider-Repositories").is_some());

    std::fs::write(fixture.config.join("layout.json"), "not json").unwrap();
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(sizes(&workspace, cx), Sizes::DEFAULT);
}

#[gpui::test]
fn the_layout_and_the_selection_come_back_after_a_restart(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (_workspace, view) = open(&fixture, cx);
    let second = point(px(400.), px(44. + 40. + 61. + 30.));
    view.simulate_click(second, Modifiers::default());
    view.run_until_parked();
    grab(view, "divider-Groups");
    move_to(view, 260., 100.);
    let_go(view);
    let place = view.debug_bounds("commits").unwrap().center();
    view.simulate_click(place, Modifiers::default());
    view.run_until_parked();

    let (restarted, view) = open(&fixture, cx);

    assert_eq!(selected(&restarted, view), Some(fixture.beta.clone()));
    assert_eq!(sizes(&restarted, view).groups, 260.);
    restarted.read_with(view, |workspace, _| {
        assert_eq!(workspace.layout.pane, Pane::Commits);
    });
}

#[gpui::test]
fn the_arrow_keys_move_the_repository_list_and_wrap_around_its_ends(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let second = point(px(400.), px(44. + 40. + 61. + 30.));
    cx.simulate_click(second, Modifiers::default());
    cx.run_until_parked();
    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));

    cx.simulate_keystrokes("down");
    assert_eq!(selected(&workspace, cx), Some(fixture.gamma.clone()));
    cx.simulate_keystrokes("down");
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));
    cx.simulate_keystrokes("up");
    assert_eq!(selected(&workspace, cx), Some(fixture.gamma.clone()));
}

#[gpui::test]
fn refresh_does_nothing_while_the_repositories_are_being_read(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.progress.reading = true;
        cx.notify();
    });
    cx.run_until_parked();

    let place = cx.debug_bounds("refresh").unwrap().center();
    cx.simulate_click(place, Modifiers::default());
    cx.run_until_parked();

    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.progress.reading, "the click started no new read");
    });
}

#[gpui::test]
fn a_repository_that_cannot_be_read_still_takes_a_row_and_counts_as_a_problem(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    std::fs::remove_dir_all(&fixture.beta).unwrap();
    let (workspace, cx) = open(&fixture, cx);

    let second = point(px(400.), px(44. + 40. + 61. + 30.));
    cx.simulate_click(second, Modifiers::default());
    cx.run_until_parked();

    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));
    workspace.read_with(cx, |workspace, _| {
        let problems = workspace
            .members()
            .iter()
            .filter(|e| e.has_problem())
            .count();
        assert_eq!(problems, 1, "only beta could not be read");
    });
}
