//! Acceptance scenarios of moving a repository to another group by dragging
//! its row onto the row of that group, driven through mouse events.

use gpui::{AppContext, Modifiers, MouseMoveEvent, MouseUpEvent, Pixels, Point, point};

use super::*;
use crate::workspace::repositories::DraggedRepository;

/// Where the middle of repository row `index` is: rows are about 61px tall,
/// below the toolbar and the header.
fn repository_row(index: u8) -> Point<Pixels> {
    point(px(400.), px(44. + 40. + 61. * f32::from(index) + 30.))
}

fn group_row(cx: &mut VisualTestContext, index: usize) -> Point<Pixels> {
    let selector = ["group-0", "group-1", "group-2"][index];
    cx.debug_bounds(selector).expect("the group row").center()
}

fn move_to(cx: &mut VisualTestContext, position: Point<Pixels>) {
    cx.simulate_event(MouseMoveEvent {
        position,
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::default(),
    });
    cx.run_until_parked();
}

/// Presses on `from` and drags to `to`, without letting go.
fn drag(cx: &mut VisualTestContext, from: Point<Pixels>, to: Point<Pixels>) {
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position: from,
        modifiers: Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    for step in [10., 30., 60.] {
        move_to(cx, point(from.x + px(step), from.y));
    }
    move_to(cx, to);
}

fn release(cx: &mut VisualTestContext, at: Point<Pixels>) {
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: at,
        modifiers: Modifiers::default(),
        click_count: 1,
    });
    cx.run_until_parked();
}

/// Drags and drops in one go.
fn drop_on(cx: &mut VisualTestContext, from: Point<Pixels>, to: Point<Pixels>) {
    drag(cx, from, to);
    release(cx, to);
}

/// The folder names each group holds, in the group order.
fn memberships(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> Vec<(String, Vec<String>)> {
    workspace.read_with(cx, |workspace, _| {
        let folder = |path: &PathBuf| path.file_name().unwrap().to_string_lossy().into_owned();
        let groups = workspace.settings.groups.iter();
        groups
            .map(|group| (group.name.clone(), group.paths.iter().map(folder).collect()))
            .collect()
    })
}

fn held(name: &str, folders: &[&str]) -> (String, Vec<String>) {
    (
        name.into(),
        folders.iter().map(ToString::to_string).collect(),
    )
}

#[gpui::test]
fn dragging_a_repository_row_onto_a_group_row_moves_it_to_that_group(cx: &mut TestAppContext) {
    let fixture = fixture(&[("a", &[0, 1, 2]), ("b", &[])]);
    let (workspace, cx) = open(&fixture, cx);

    let target = group_row(cx, 1);
    drop_on(cx, repository_row(1), target);

    let groups = memberships(&workspace, cx);
    assert_eq!(
        groups,
        [held("a", &["alpha", "gamma"]), held("b", &["beta"])]
    );
    assert_eq!(names(&workspace, cx), ["alpha", "gamma"]);
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("a"));
        assert_eq!(workspace.selection.repository, Some(fixture.alpha.clone()));
    });
    assert!(fixture.beta.join("README.md").exists());
}

#[gpui::test]
fn a_moved_repository_stays_in_its_new_group_after_a_restart(cx: &mut TestAppContext) {
    let fixture = fixture(&[("a", &[0, 1]), ("b", &[2])]);
    let (_, cx) = open(&fixture, cx);
    let target = group_row(cx, 1);
    drop_on(cx, repository_row(0), target);

    let config = fixture.config.clone();
    let reopened = cx.update(|_, cx| cx.new(|cx| Workspace::new(&config, cx)));
    cx.run_until_parked();

    let groups = memberships(&reopened, cx);
    assert_eq!(
        groups,
        [held("a", &["beta"]), held("b", &["gamma", "alpha"])]
    );
    assert!(fixture.alpha.join("README.md").exists());
}

#[gpui::test]
fn dragging_a_repository_over_a_group_row_marks_the_row(cx: &mut TestAppContext) {
    let fixture = fixture(&[("a", &[0, 1]), ("b", &[])]);
    let (workspace, cx) = open(&fixture, cx);

    let other = group_row(cx, 1);
    drag(cx, repository_row(0), other);

    assert!(cx.update(|_, cx| cx.has_active_drag()));
    // GPUI cannot report painted styles, so the rule behind the border is
    // asserted directly: the source group is no target, any other is.
    let dragged = DraggedRepository {
        path: fixture.alpha.clone(),
        name: "alpha".into(),
        group: "a".into(),
    };
    assert!(dragged.moves_onto("b"));
    assert!(!dragged.moves_onto("a"));
    let source = group_row(cx, 0);
    move_to(cx, source);
    release(cx, source);
    assert_eq!(
        memberships(&workspace, cx)[0],
        held("a", &["alpha", "beta"])
    );
}

#[gpui::test]
fn moving_the_selected_repository_out_selects_the_next_row(cx: &mut TestAppContext) {
    let fixture = fixture(&[("a", &[0, 1, 2]), ("b", &[])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(fixture.beta.clone()), cx);
    });
    let target = group_row(cx, 1);

    // The middle row leaves: the row after it is selected.
    drop_on(cx, repository_row(1), target);
    assert_eq!(selected(&workspace, cx), Some(fixture.gamma.clone()));
    // The last row leaves: the row before it is selected.
    drop_on(cx, repository_row(1), target);
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));
    // The only row leaves: nothing is selected.
    drop_on(cx, repository_row(0), target);
    assert_eq!(selected(&workspace, cx), None);
    assert_eq!(memberships(&workspace, cx)[0], held("a", &[]));
}

#[gpui::test]
fn a_group_row_dropped_on_a_group_row_still_reorders_and_a_file_row_does_not_move_a_repository(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[("a", &[0, 1]), ("b", &[]), ("c", &[])]);
    let (workspace, cx) = open(&fixture, cx);

    let (first, third) = (group_row(cx, 0), group_row(cx, 2));
    drop_on(cx, third, first);
    let order: Vec<String> = memberships(&workspace, cx)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    assert_eq!(order, ["c", "a", "b"]);

    // beta has an untracked file; its row dropped on a group row goes nowhere.
    workspace.update(cx, |workspace, cx| {
        workspace.select_repository(Some(fixture.beta.clone()), cx);
    });
    cx.run_until_parked();
    let file = cx
        .debug_bounds("label-unstaged-draft.txt")
        .expect("the file row");
    let target = group_row(cx, 2);
    drop_on(cx, file.center(), target);
    assert_eq!(
        memberships(&workspace, cx)[1],
        held("a", &["alpha", "beta"])
    );
    assert_eq!(
        git_output(&fixture.beta, &["status", "--porcelain"]),
        "?? draft.txt\n"
    );
}
