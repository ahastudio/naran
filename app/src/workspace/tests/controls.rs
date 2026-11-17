//! The buttons, links, keys and mouse presses of the window, each used the
//! way a person uses it.

use gpui::{Modifiers, WeakEntity};

use super::*;

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let place = cx.debug_bounds(selector).expect(selector).center();
    cx.simulate_click(place, Modifiers::default());
    cx.run_until_parked();
}

fn press(cx: &mut VisualTestContext, selector: &'static str, button: MouseButton) {
    let position = cx.debug_bounds(selector).expect(selector).center();
    cx.simulate_event(MouseDownEvent {
        button,
        position,
        modifiers: Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    cx.run_until_parked();
}

fn pane(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) -> Pane {
    workspace.read_with(cx, |workspace, _| workspace.layout.pane)
}

#[gpui::test]
fn pressing_the_group_column_and_a_group_moves_the_keys_there(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0]), ("home", &[1])]);
    let (workspace, cx) = open(&fixture, cx);

    click(cx, "group-1");

    assert_eq!(pane(&workspace, cx), Pane::Groups);
    assert_eq!(names(&workspace, cx), ["beta"]);
}

#[gpui::test]
fn the_arrow_keys_do_nothing_in_a_list_that_is_empty(cx: &mut TestAppContext) {
    let fixture = fixture(&[("empty", &[])]);
    let (workspace, cx) = open(&fixture, cx);

    cx.simulate_keystrokes("down");
    assert_eq!(selected(&workspace, cx), None);

    let fixture = fixture_with(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, _| workspace.layout.pane = Pane::Groups);
    cx.simulate_keystrokes("down");
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group, None);
    });
}

#[gpui::test]
fn shift_arrows_outside_the_file_lists_pick_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let before = selected(&workspace, cx);

    cx.simulate_keystrokes("shift-down shift-up");

    assert_eq!(pane(&workspace, cx), Pane::Repositories);
    assert_eq!(selected(&workspace, cx), before);
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.shown.marked.paths.is_empty());
    });
}

#[gpui::test]
fn refresh_reads_again_from_the_button_and_from_the_keys(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    std::fs::write(fixture.beta.join("new.txt"), "new").unwrap();

    click(cx, "refresh");
    cx.simulate_keystrokes("cmd-r");
    cx.run_until_parked();

    workspace.read_with(cx, |workspace, _| {
        assert!(!workspace.progress.reading);
        let beta = workspace
            .entries
            .iter()
            .find(|entry| entry.path() == fixture.beta);
        let untracked = |entry: &Entry| match entry {
            Entry::Ready(summary) => summary.status.untracked,
            Entry::Failed { .. } => 0,
        };
        assert_eq!(beta.map(untracked), Some(2), "draft.txt and new.txt");
    });
}

#[gpui::test]
fn copy_puts_the_picked_text_on_the_clipboard(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, _| {
        workspace.texts.info.set_lines(vec!["copied".into()]);
        workspace.texts.select_all();
    });

    cx.simulate_keystrokes("cmd-c");

    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("copied"));
}

#[gpui::test]
fn pressing_a_divider_starts_dragging_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    press(cx, "divider-Groups", MouseButton::Left);

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.interaction.dragging, Some(Divider::Groups));
    });
}

#[gpui::test]
fn the_links_start_a_new_group_and_add_repositories(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, _| {
        workspace.services.picker = crate::picker::Picker::new("true").marked_by("unused");
    });

    click(cx, "add-repository");
    workspace.read_with(cx, |workspace, _| {
        assert!(!workspace.progress.picking_folders);
    });
    click(cx, "new-group");

    workspace.read_with(cx, |workspace, _| {
        let editing = workspace
            .interaction
            .editing
            .as_ref()
            .map(|editing| &editing.kind);
        assert!(matches!(editing, Some(EditKind::NewGroup)));
    });
}

#[gpui::test]
fn a_notice_goes_away_when_it_is_clicked(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.report("It failed", Outcome::Failed);
        cx.notify();
    });
    cx.run_until_parked();

    click(cx, "notice");

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.feedback.notice, None);
    });
}

#[gpui::test]
fn finder_that_cannot_start_is_reported(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    workspace.update(cx, |workspace, _| {
        workspace.services.finder = crate::finder::Finder::new("naran-no-such-program");
        workspace.reveal(&fixture.alpha);
    });

    workspace.read_with(cx, |workspace, _| {
        let notice = workspace.feedback.notice.clone().unwrap_or_default();
        assert!(
            notice.starts_with("Could not show the folder in Finder"),
            "{notice}"
        );
    });
}

#[gpui::test]
fn pressing_beside_the_menu_closes_it_and_pressing_on_it_does_not(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    let open_menu = |workspace: &Entity<Workspace>, cx: &mut VisualTestContext| {
        workspace.update(cx, |workspace, cx| {
            let target = MenuTarget::Group("work".into());
            workspace.interaction.menu = Some(ContextMenu {
                target,
                position: gpui::point(px(40.), px(40.)),
            });
            cx.notify();
        });
        cx.run_until_parked();
    };
    let is_open = |workspace: &Entity<Workspace>, cx: &mut VisualTestContext| {
        workspace.read_with(cx, |workspace, _| workspace.interaction.menu.is_some())
    };

    open_menu(&workspace, cx);
    right_click(cx, 60., 58.);
    assert!(is_open(&workspace, cx), "a press on the menu stays on it");
    press(cx, "group-menu-cover", MouseButton::Right);
    assert!(!is_open(&workspace, cx));
    open_menu(&workspace, cx);
    press(cx, "group-menu-cover", MouseButton::Left);
    assert!(!is_open(&workspace, cx));
}

#[gpui::test]
fn changes_stop_being_read_once_the_window_is_gone(cx: &mut TestAppContext) {
    let (sender, receiver) = futures::channel::mpsc::unbounded();
    sender.unbounded_send(vec![PathBuf::from("/gone")]).unwrap();

    let done = std::rc::Rc::new(std::cell::Cell::new(false));
    let finished = done.clone();
    cx.update(|cx| {
        cx.spawn(async move |cx| {
            forward_changes(receiver, WeakEntity::new_invalid(), cx).await;
            finished.set(true);
        })
        .detach();
    });
    cx.run_until_parked();

    assert!(done.get(), "the loop ends while the sender is still there");
    drop(sender);
}

#[gpui::test]
fn the_window_title_is_the_selected_repository(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let title = |workspace: &Entity<Workspace>, cx: &mut VisualTestContext| {
        workspace.read_with(cx, |workspace, _| workspace.title.clone())
    };
    assert_eq!(title(&workspace, cx), "alpha");
    assert!(
        cx.debug_bounds("window-title").is_some(),
        "drawn in the toolbar"
    );

    cx.simulate_keystrokes("down");
    assert_eq!(title(&workspace, cx), "beta");

    workspace.update(cx, |workspace, cx| workspace.select_repository(None, cx));
    cx.run_until_parked();
    assert_eq!(title(&workspace, cx), "Naran");
}
