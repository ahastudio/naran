//! The context menu and the confirmations it opens.

use super::*;

#[gpui::test]
fn removing_a_repository_asks_first_and_cancel_changes_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1])]);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(names(&workspace, cx), ["alpha", "beta"]);

    cx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.confirm_remove(fixture.alpha.clone(), window, cx);
        });
    });
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    assert_eq!(
        names(&workspace, cx),
        ["alpha", "beta"],
        "Cancel keeps everything"
    );

    cx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.confirm_remove(fixture.alpha.clone(), window, cx);
        });
    });
    cx.simulate_prompt_answer("Remove");
    cx.run_until_parked();
    assert_eq!(names(&workspace, cx), ["beta"]);
    assert!(fixture.alpha.exists(), "the folder itself is untouched");
}

#[gpui::test]
fn deleting_a_group_asks_first_and_cancel_changes_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1]), ("home", &[1, 2])]);
    let (workspace, cx) = open(&fixture, cx);

    cx.update(|window, cx| {
        workspace.update(cx, |_, cx| {
            Workspace::delete_group("home".into(), window, cx);
        });
    });
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        let groups: Vec<&str> = workspace
            .settings
            .groups
            .iter()
            .map(|group| group.name.as_str())
            .collect();
        assert_eq!(groups, ["work", "home"]);
    });

    // Deleting a group that is not the selected one keeps the selection.
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("work"));
    });
    cx.update(|window, cx| {
        workspace.update(cx, |_, cx| {
            Workspace::delete_group("home".into(), window, cx);
        });
    });
    cx.simulate_prompt_answer("Delete");
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        let groups: Vec<&str> = workspace
            .settings
            .groups
            .iter()
            .map(|group| group.name.as_str())
            .collect();
        assert_eq!(groups, ["work"]);
        assert_eq!(workspace.selection.group.as_deref(), Some("work"));
    });
}

#[gpui::test]
fn the_menu_item_delete_opens_the_confirmation(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1]), ("home", &[1, 2])]);
    let (workspace, cx) = open(&fixture, cx);

    // The second group row is near the top of the left column.
    let mut opened = None;
    for y in (60..140_u16).step_by(6) {
        right_click(cx, 60., f32::from(y));
        let menu = workspace.read_with(cx, |workspace, _| workspace.interaction.menu.clone());
        if let Some(menu) = menu {
            if matches!(&menu.target, MenuTarget::Group(name) if name == "home") {
                opened = Some(menu);
                break;
            }
            // Close it and look further down.
            right_click(cx, 600., 600.);
        }
    }
    let menu = opened.expect("a right click on the group opens its menu");

    // Delete… is the second item of the menu.
    cx.simulate_click(
        gpui::point(menu.position.x + px(20.), menu.position.y + px(46.)),
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();
    assert!(
        cx.has_pending_prompt(),
        "pressing Delete… asks for confirmation"
    );
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.settings.groups.len(), 2);
    });
}

#[gpui::test]
fn the_menu_item_remove_opens_the_confirmation(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1])]);
    let (workspace, cx) = open(&fixture, cx);

    let mut opened = None;
    for y in (80..200_u16).step_by(6) {
        right_click(cx, 400., f32::from(y));
        let menu = workspace.read_with(cx, |workspace, _| workspace.interaction.menu.clone());
        if let Some(menu) = menu {
            if matches!(&menu.target, MenuTarget::Repository(_)) {
                opened = Some(menu);
                break;
            }
            right_click(cx, 900., 900.);
        }
    }
    let menu = opened.expect("a right click on a repository opens its menu");

    cx.simulate_click(
        gpui::point(menu.position.x + px(20.), menu.position.y + px(46.)),
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();
    assert!(
        cx.has_pending_prompt(),
        "pressing Remove… asks for confirmation"
    );
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    assert_eq!(names(&workspace, cx), ["alpha", "beta"]);
}

#[gpui::test]
fn reveal_in_finder_closes_the_menu_and_asks_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, _| {
        workspace.services.finder = crate::finder::Finder::new("true");
    });

    let mut opened = None;
    for y in (80..200_u16).step_by(6) {
        right_click(cx, 400., f32::from(y));
        let menu = workspace.read_with(cx, |workspace, _| workspace.interaction.menu.clone());
        if let Some(menu) = menu {
            if matches!(&menu.target, MenuTarget::Repository(_)) {
                opened = Some(menu);
                break;
            }
            right_click(cx, 900., 900.);
        }
    }
    let menu = opened.expect("a right click on a repository opens its menu");

    // Reveal in Finder is the first item.
    cx.simulate_click(
        gpui::point(menu.position.x + px(20.), menu.position.y + px(18.)),
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.interaction.menu.is_none(), "the menu closes");
        assert!(
            workspace.services.revealed.is_some(),
            "the folder is shown in Finder"
        );
    });
    assert!(!cx.has_pending_prompt(), "revealing needs no confirmation");
    assert_eq!(names(&workspace, cx), ["alpha", "beta"]);
}

#[gpui::test]
fn the_menu_item_rename_starts_editing_the_group_name(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1])]);
    let (workspace, cx) = open(&fixture, cx);

    let mut opened = None;
    for y in (50..200_u16).step_by(6) {
        right_click(cx, 60., f32::from(y));
        let menu = workspace.read_with(cx, |workspace, _| workspace.interaction.menu.clone());
        if let Some(menu) = menu {
            if matches!(&menu.target, MenuTarget::Group(_)) {
                opened = Some(menu);
                break;
            }
            right_click(cx, 900., 900.);
        }
    }
    let menu = opened.expect("a right click on a group opens its menu");

    // Rename is the first item.
    cx.simulate_click(
        gpui::point(menu.position.x + px(20.), menu.position.y + px(18.)),
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();

    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.interaction.menu.is_none());
        let editing = workspace
            .interaction
            .editing
            .as_ref()
            .expect("the name is being edited");
        assert!(matches!(&editing.kind, EditKind::RenameGroup(name) if name == "work"));
    });
}
