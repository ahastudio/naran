//! Acceptance scenarios of the groups and repositories feature, driven
//! through clicks, keystrokes and the notices on screen.

use gpui::AppContext;

use super::*;

fn type_name(cx: &mut VisualTestContext, name: &str) {
    click(cx, "new-group");
    cx.simulate_input(name);
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
}

pub(super) fn group_names(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> Vec<String> {
    workspace.read_with(cx, |workspace, _| {
        let groups = workspace.settings.groups.iter();
        groups.map(|group| group.name.clone()).collect()
    })
}

#[gpui::test]
fn clicking_new_group_and_typing_a_name_creates_and_selects_the_group(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);

    type_name(cx, "home");

    assert_eq!(group_names(&workspace, cx), ["work", "home"]);
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("home"));
    });
}

#[gpui::test]
fn a_taken_name_shows_a_red_notice_that_a_click_dismisses(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);

    type_name(cx, "work");

    assert_eq!(group_names(&workspace, cx), ["work"]);
    assert!(
        cx.debug_bounds("notice").is_some(),
        "the notice is on screen"
    );
    click(cx, "notice");
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.feedback.notice.is_none(), "a click dismisses it");
    });
}

#[gpui::test]
fn an_empty_name_changes_nothing_and_shows_no_notice(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);

    type_name(cx, "   ");

    assert_eq!(group_names(&workspace, cx), ["work"]);
    assert!(cx.debug_bounds("notice").is_none());
}

#[gpui::test]
fn a_folder_that_is_not_a_repository_is_listed_in_a_notice_on_screen(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    let plain = fixture.alpha.parent().unwrap().join("plain");
    std::fs::create_dir_all(&plain).unwrap();

    workspace.update(cx, |workspace, cx| {
        workspace.add_repositories(vec![plain], cx);
    });
    cx.run_until_parked();

    assert_eq!(names(&workspace, cx), ["alpha"]);
    assert!(
        cx.debug_bounds("notice").is_some(),
        "the notice is on screen"
    );
}

#[gpui::test]
fn without_a_group_adding_a_folder_asks_to_create_one_first(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, _| {
        workspace.settings.groups.clear();
        workspace.selection.group = None;
    });

    workspace.update(cx, |workspace, cx| {
        workspace.add_repositories(vec![fixture.beta.clone()], cx);
    });
    cx.run_until_parked();

    workspace.read_with(cx, |workspace, _| {
        let notice = workspace.feedback.notice.clone().unwrap();
        assert!(notice.starts_with("Create a group first"), "{notice}");
    });
    assert!(cx.debug_bounds("notice").is_some());
}

#[gpui::test]
fn a_new_window_restores_the_last_group_and_repository(cx: &mut TestAppContext) {
    let fixture = fixture(&[("home", &[2]), ("work", &[0, 1])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_group(Some("work".into()), cx);
    });
    workspace.update(cx, |workspace, cx| workspace.step_selection(1, cx));
    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));

    let config = fixture.config.clone();
    let reopened = cx.update(|_, cx| cx.new(|cx| Workspace::new(&config, cx)));
    cx.run_until_parked();

    reopened.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("work"));
        assert_eq!(workspace.selection.repository, Some(fixture.beta.clone()));
    });
}

#[gpui::test]
fn a_new_window_restores_an_empty_group(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1]), ("empty", &[])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_group(Some("empty".into()), cx);
    });

    let config = fixture.config.clone();
    let reopened = cx.update(|_, cx| cx.new(|cx| Workspace::new(&config, cx)));
    cx.run_until_parked();

    reopened.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("empty"));
    });
}

#[gpui::test]
fn switching_to_a_group_that_holds_the_same_repository_is_remembered(cx: &mut TestAppContext) {
    let fixture = fixture(&[("home", &[0, 2]), ("work", &[0, 1])]);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));
    workspace.update(cx, |workspace, cx| {
        workspace.select_group(Some("work".into()), cx);
    });
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));

    let config = fixture.config.clone();
    let reopened = cx.update(|_, cx| cx.new(|cx| Workspace::new(&config, cx)));
    cx.run_until_parked();

    reopened.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("work"));
        assert_eq!(workspace.selection.repository, Some(fixture.alpha.clone()));
    });
}

#[gpui::test]
fn a_group_name_is_saved_without_the_spaces_around_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);

    type_name(cx, "  home  ");

    assert_eq!(group_names(&workspace, cx), ["work", "home"]);
}

/// Right-clicks down the group column until the menu of `group` opens.
fn open_group_menu(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
    group: &str,
) -> ContextMenu {
    for y in (50..200_u16).step_by(6) {
        right_click(cx, 60., f32::from(y));
        let menu = workspace.read_with(cx, |workspace, _| workspace.interaction.menu.clone());
        match menu {
            Some(menu) if matches!(&menu.target, MenuTarget::Group(name) if name == group) => {
                return menu;
            }
            Some(_) => right_click(cx, 900., 900.),
            None => {}
        }
    }
    panic!("no menu of {group}");
}

fn rename_from_menu(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
    group: &str,
    name: &str,
) {
    let menu = open_group_menu(workspace, cx, group);
    let item = gpui::point(menu.position.x + px(20.), menu.position.y + px(18.));
    cx.simulate_click(item, gpui::Modifiers::default());
    cx.run_until_parked();
    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input(name);
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
}

#[gpui::test]
fn renaming_from_the_menu_to_a_taken_or_blank_name_changes_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0]), ("home", &[1])]);
    let (workspace, cx) = open(&fixture, cx);

    rename_from_menu(&workspace, cx, "work", "   ");
    assert_eq!(group_names(&workspace, cx), ["work", "home"]);
    assert!(
        cx.debug_bounds("notice").is_none(),
        "a blank name is silent"
    );

    rename_from_menu(&workspace, cx, "work", "home");
    assert_eq!(group_names(&workspace, cx), ["work", "home"]);
    assert!(cx.debug_bounds("notice").is_some(), "a taken name is told");
}

/// A program that prints the given folders, one per line, like the picker.
fn picker_printing(folders: &[&Path], directory: &Path) -> crate::picker::Picker {
    use std::os::unix::fs::PermissionsExt;
    let lines: Vec<String> = folders
        .iter()
        .map(|path| format!("echo '{}/'", path.display()))
        .collect();
    let script = directory.join("pick.sh");
    std::fs::write(&script, format!("#!/bin/sh\n{}\n", lines.join("\n"))).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    crate::picker::Picker::new(script.to_str().unwrap()).marked_by("unused")
}

#[gpui::test]
fn picking_a_repository_the_group_already_holds_keeps_it_once(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);
    let picker = picker_printing(&[&fixture.alpha], fixture.config.as_path());
    workspace.update(cx, |workspace, _| workspace.services.picker = picker);

    cx.simulate_keystrokes("cmd-o");
    cx.run_until_parked();

    assert_eq!(names(&workspace, cx), ["alpha"]);
    let saved: Settings = store::load(&fixture.config.join("settings.json"));
    assert_eq!(saved.group("work").unwrap().paths.len(), 1);
}

#[gpui::test]
fn broken_saved_files_start_the_app_with_defaults(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    std::fs::write(fixture.config.join("selection.json"), "not json").unwrap();
    let (workspace, cx) = open(&fixture, cx);

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("work"));
    });
    assert_eq!(names(&workspace, cx), ["alpha"]);
    std::fs::write(fixture.config.join("settings.json"), "not json").unwrap();
    let config = fixture.config.clone();
    let reopened = cx.update(|_, cx| cx.new(|cx| Workspace::new(&config, cx)));
    cx.run_until_parked();
    reopened.read_with(cx, |workspace, _| {
        assert_eq!(workspace.settings.groups.len(), 0);
    });
}
