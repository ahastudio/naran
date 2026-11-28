//! Groups: choosing, naming, filling and deleting them.

use super::*;

#[gpui::test]
fn reads_every_repository_and_selects_the_first(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    assert_eq!(names(&workspace, cx), ["alpha", "beta", "gamma"]);
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));
    workspace.read_with(cx, |workspace, _| {
        let detail = workspace
            .shown
            .detail
            .as_ref()
            .expect("details of the selection");
        assert_eq!(detail.history.as_ref().unwrap()[0].subject, "first");
        let beta = workspace
            .entries
            .iter()
            .find(|entry| entry.path() == fixture.beta)
            .unwrap();
        assert_eq!(model::describe(beta).0, "Changes");
    });
}

#[gpui::test]
fn selecting_a_group_shows_only_its_repositories_and_keeps_the_choice(cx: &mut TestAppContext) {
    let fixture = fixture(&[("home", &[0]), ("work", &[2, 1])]);
    let (workspace, cx) = open(&fixture, cx);
    assert_eq!(
        names(&workspace, cx),
        ["alpha"],
        "the first group is selected at start"
    );

    workspace.update(cx, |workspace, cx| {
        workspace.select_group(Some("work".into()), cx);
    });
    cx.run_until_parked();

    assert_eq!(names(&workspace, cx), ["gamma", "beta"]);
    assert_eq!(selected(&workspace, cx), Some(fixture.gamma.clone()));
    let saved: Selection = store::load(&fixture.config.join("selection.json"));
    assert_eq!(saved.group.as_deref(), Some("work"));
    assert_eq!(saved.repository, Some(fixture.gamma.clone()));

    workspace.update(cx, |workspace, cx| workspace.step_selection(1, cx));
    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));
    cx.simulate_keystrokes("down");
    assert_eq!(selected(&workspace, cx), Some(fixture.gamma.clone()));
    cx.simulate_keystrokes("up");
    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));
}

#[gpui::test]
fn creates_and_renames_a_group_by_typing(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);

    cx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.start_editing(EditKind::NewGroup, window, cx);
        });
    });
    cx.simulate_input("회사 서비스");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();

    let settings: Settings = store::load(&fixture.config.join("settings.json"));
    assert_eq!(settings.groups.last().unwrap().name, "회사 서비스");
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.interaction.editing.is_none());
        assert_eq!(workspace.selection.group.as_deref(), Some("회사 서비스"));
    });
    assert_eq!(names(&workspace, cx), [] as [String; 0]);

    cx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.start_editing(EditKind::RenameGroup("회사 서비스".into()), window, cx);
        });
    });
    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("업무");
    cx.simulate_keystrokes("enter");
    let settings: Settings = store::load(&fixture.config.join("settings.json"));
    assert_eq!(settings.groups.last().unwrap().name, "업무");

    cx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.start_editing(EditKind::NewGroup, window, cx);
        });
    });
    cx.simulate_input("버릴 이름");
    cx.simulate_keystrokes("escape");
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.interaction.editing.is_none());
    });
    let settings: Settings = store::load(&fixture.config.join("settings.json"));
    assert_eq!(settings.groups.len(), 2);
}

#[gpui::test]
fn refuses_a_name_that_is_already_taken(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[])]);
    let (workspace, cx) = open(&fixture, cx);

    cx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.start_editing(EditKind::NewGroup, window, cx);
        });
    });
    cx.simulate_input("work");
    cx.simulate_keystrokes("enter");

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.settings.groups.len(), 1);
        let (message, outcome) = workspace.feedback.message.clone().unwrap();
        assert_eq!(outcome, Outcome::Failed);
        assert!(message.contains("already exists"));
    });
}

#[gpui::test]
fn puts_repositories_into_a_group_and_takes_them_out(cx: &mut TestAppContext) {
    let fixture = fixture(&[("other", &[0, 1, 2]), ("work", &[])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.select_group(Some("work".into()), cx);
    });

    // Repositories Naran already knows go into the group without being read again.
    workspace.update(cx, |workspace, cx| {
        workspace.add_repositories(vec![fixture.beta.clone()], cx);
    });
    cx.run_until_parked();
    workspace.update(cx, |workspace, cx| {
        workspace.add_repositories(vec![fixture.alpha.clone()], cx);
    });
    cx.run_until_parked();
    assert_eq!(names(&workspace, cx), ["beta", "alpha"]);
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));

    workspace.update(cx, |workspace, cx| {
        workspace.remove(&fixture.alpha, cx);
    });
    cx.run_until_parked();
    assert_eq!(names(&workspace, cx), ["beta"]);
    assert_eq!(selected(&workspace, cx), Some(fixture.beta.clone()));
    let paths: Vec<PathBuf> = store::load(&fixture.config.join("repositories.json"));
    assert_eq!(paths.len(), 3, "still held by the other group");
}

#[gpui::test]
fn drops_a_repository_that_leaves_its_last_group(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open_watched(&fixture, cx);

    workspace.update(cx, |workspace, cx| {
        workspace.remove(&fixture.alpha, cx);
    });
    cx.run_until_parked();

    assert_eq!(names(&workspace, cx), ["beta", "gamma"]);
    let paths: Vec<PathBuf> = store::load(&fixture.config.join("repositories.json"));
    assert_eq!(paths, [fixture.beta.clone(), fixture.gamma.clone()]);
    assert!(fixture.alpha.exists(), "the folder itself is untouched");
}

#[gpui::test]
fn deleting_a_group_drops_the_repositories_only_it_held(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0, 1]), ("home", &[1, 2])]);
    let (workspace, cx) = open(&fixture, cx);

    cx.update(|window, cx| {
        workspace.update(cx, |_, cx| {
            Workspace::delete_group("work".into(), window, cx);
        });
    });
    cx.simulate_prompt_answer("Delete");
    cx.run_until_parked();

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("home"));
    });
    assert_eq!(names(&workspace, cx), ["beta", "gamma"]);
    let paths: Vec<PathBuf> = store::load(&fixture.config.join("repositories.json"));
    assert_eq!(paths, [fixture.beta.clone(), fixture.gamma.clone()]);
}

#[gpui::test]
fn without_groups_nothing_is_listed_and_nothing_can_be_added(cx: &mut TestAppContext) {
    let fixture = fixture_with(&[]);
    let (workspace, cx) = open(&fixture, cx);

    assert_eq!(names(&workspace, cx), [] as [String; 0]);
    let paths: Vec<PathBuf> = store::load(&fixture.config.join("repositories.json"));
    assert!(
        paths.is_empty(),
        "repositories outside every group are dropped at start"
    );

    workspace.update(cx, |workspace, cx| {
        workspace.add_repositories(vec![fixture.alpha.clone()], cx);
    });
    cx.run_until_parked();
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.paths, [] as [PathBuf; 0]);
        let (message, outcome) = workspace.feedback.message.clone().unwrap();
        assert_eq!(outcome, Outcome::Failed);
        assert!(message.starts_with("Create a group first"));
    });
}

#[gpui::test]
fn forgets_a_saved_group_that_no_longer_exists(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    store::save(
        &fixture.config.join("selection.json"),
        &Selection {
            group: Some("gone".into()),
            repository: Some(fixture.gamma.clone()),
        },
    )
    .unwrap();
    let (workspace, cx) = open(&fixture, cx);

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("기본"));
    });
    assert_eq!(selected(&workspace, cx), Some(fixture.gamma.clone()));
}

#[gpui::test]
fn the_arrow_keys_move_through_the_groups(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0]), ("home", &[1])]);
    let (workspace, cx) = open(&fixture, cx);
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("work"));
    });

    workspace.update(cx, |workspace, cx| {
        workspace.layout.pane = Pane::Groups;
        workspace.step(1, cx);
    });

    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("home"));
    });
}

#[gpui::test]
fn an_empty_name_ends_editing_and_says_nothing(cx: &mut TestAppContext) {
    let fixture = fixture(&[("work", &[0])]);
    let (workspace, cx) = open(&fixture, cx);

    cx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.start_editing(EditKind::NewGroup, window, cx);
            workspace.finish_editing("   ", window, cx);
        });
    });

    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.interaction.editing.is_none());
        assert!(workspace.feedback.message.is_none());
        assert_eq!(workspace.settings.groups.len(), 1);
    });
}

#[gpui::test]
fn dropping_a_group_on_another_group_changes_the_order_and_keeps_it(cx: &mut TestAppContext) {
    let fixture = fixture(&[("a", &[0]), ("b", &[1]), ("c", &[2])]);
    let (workspace, cx) = open(&fixture, cx);
    let group_names = |workspace: &Entity<Workspace>, cx: &mut VisualTestContext| -> Vec<String> {
        workspace.read_with(cx, |workspace, _| {
            let groups = workspace.settings.groups.iter();
            groups.map(|group| group.name.clone()).collect()
        })
    };
    assert_eq!(group_names(&workspace, cx), ["a", "b", "c"]);

    workspace.update(cx, |workspace, cx| workspace.move_group("c", "a", cx));
    assert_eq!(group_names(&workspace, cx), ["c", "a", "b"]);
    let settings: Settings = store::load(&fixture.config.join("settings.json"));
    let saved: Vec<&str> = settings.groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(saved, ["c", "a", "b"]);
    workspace.read_with(cx, |workspace, _| {
        assert_eq!(workspace.selection.group.as_deref(), Some("a"));
    });
    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));

    // Dropped on itself or on a name that is not a group: nothing changes.
    workspace.update(cx, |workspace, cx| {
        workspace.move_group("a", "a", cx);
        workspace.move_group("a", "nowhere", cx);
    });
    assert_eq!(group_names(&workspace, cx), ["c", "a", "b"]);
}
