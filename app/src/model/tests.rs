use naran_core::Status;

use super::*;

mod diff;
mod groups;
mod labels;
mod worktrees;

fn settings() -> Settings {
    Settings {
        groups: vec![
            Group {
                name: "work".into(),
                paths: vec!["/a".into(), "/b".into()],
            },
            Group {
                name: "home".into(),
                paths: vec!["/b".into()],
            },
        ],
    }
}

fn ready(path: &str, status: Status) -> Entry {
    Entry::Ready(Summary {
        path: path.into(),
        name: path.trim_start_matches('/').into(),
        status,
        operation: None,
        worktrees: Vec::new(),
    })
}

fn doing(path: &str, operation: Operation) -> Entry {
    Entry::Ready(Summary {
        path: path.into(),
        name: path.trim_start_matches('/').into(),
        status: Status::default(),
        operation: Some(operation),
        worktrees: Vec::new(),
    })
}

fn failed(path: &str) -> Entry {
    Entry::Failed {
        path: path.into(),
        error: Error::Timeout {
            command: "status".into(),
        },
    }
}

#[test]
fn creates_groups_with_new_non_empty_names_only() {
    let mut settings = settings();

    assert!(settings.create_group(" new "));
    assert_eq!(settings.groups.last().unwrap().name, "new");
    assert!(!settings.create_group(""));
    assert!(!settings.create_group("work"));
    assert_eq!(settings.groups.len(), 3);
}

#[test]
fn renames_unless_the_name_is_empty_or_taken() {
    let mut settings = settings();

    assert!(!settings.rename_group("work", "home"));
    assert!(!settings.rename_group("work", " "));
    assert!(!settings.rename_group("missing", "other"));
    assert!(settings.rename_group("work", "office"));
    assert_eq!(settings.groups[0].name, "office");
}

#[test]
fn adds_and_removes_repositories() {
    let mut settings = settings();

    settings.add_to_group("home", Path::new("/c"));
    settings.add_to_group("home", Path::new("/c"));
    assert_eq!(settings.groups[1].paths, [PathBuf::from("/b"), "/c".into()]);

    settings.remove_from_group("work", Path::new("/a"));
    assert_eq!(settings.groups[0].paths, [PathBuf::from("/b")]);
    settings.remove_from_group("nowhere", Path::new("/b"));
    assert_eq!(
        settings.groups[0].paths,
        [PathBuf::from("/b")],
        "an unknown group changes nothing"
    );

    settings.delete_group("work");
    assert_eq!(settings.groups.len(), 1);
}

#[test]
fn lists_members_in_group_order_and_skips_unknown_paths() {
    let entries = vec![failed("/a"), failed("/b"), failed("/c")];
    let group = Group {
        name: "work".into(),
        paths: vec!["/c".into(), "/missing".into(), "/a".into()],
    };

    let names: Vec<_> = members(&entries, &group)
        .iter()
        .map(|entry| entry.path())
        .collect();
    assert_eq!(names, [Path::new("/c"), Path::new("/a")]);
}

#[test]
fn counts_changes_and_spots_problems() {
    let changed = ready(
        "/a",
        Status {
            staged: 1,
            unstaged: 2,
            untracked: 3,
            ..Status::default()
        },
    );
    let conflicted = ready(
        "/b",
        Status {
            conflicted: 1,
            ..Status::default()
        },
    );

    assert_eq!(changed.change_count(), 6);
    assert!(!changed.has_problem());
    assert!(conflicted.has_problem());
    assert!(failed("/c").has_problem());
    assert_eq!(failed("/c").change_count(), 0);
    assert_eq!(failed("/c").name(), "/c");
}

#[test]
fn describes_the_most_urgent_state_first() {
    let text = |entry: &Entry| describe(entry).0;
    let with = |status: Status| ready("/a", status);
    let upstream = || Some("origin/main".to_string());

    assert_eq!(
        text(&with(Status {
            upstream: upstream(),
            ..Status::default()
        })),
        "Clean"
    );
    assert_eq!(text(&with(Status::default())), "No upstream");
    assert_eq!(
        text(&with(Status {
            behind: 1,
            ..Status::default()
        })),
        "Behind"
    );
    assert_eq!(
        text(&with(Status {
            ahead: 1,
            ..Status::default()
        })),
        "Ahead"
    );
    assert_eq!(
        text(&with(Status {
            ahead: 1,
            behind: 1,
            ..Status::default()
        })),
        "Diverged"
    );
    assert_eq!(
        text(&with(Status {
            untracked: 1,
            ahead: 1,
            ..Status::default()
        })),
        "Changes"
    );
    assert_eq!(
        describe(&with(Status {
            conflicted: 1,
            ..Status::default()
        })),
        ("Conflicts".into(), Tone::Problem)
    );

    for (operation, text) in [
        (Operation::Merge, "Merging"),
        (Operation::Rebase, "Rebasing"),
        (Operation::CherryPick, "Cherry-picking"),
        (Operation::Revert, "Reverting"),
    ] {
        assert_eq!(
            describe(&doing("/a", operation)),
            (text.into(), Tone::Problem)
        );
    }
    assert_eq!(
        describe(&failed("/b")),
        ("git status timed out".into(), Tone::Problem)
    );
}

#[test]
fn describes_elapsed_time() {
    assert_eq!(elapsed(-5), "just now");
    assert_eq!(elapsed(59), "just now");
    assert_eq!(elapsed(60), "1 minute ago");
    assert_eq!(elapsed(120), "2 minutes ago");
    assert_eq!(elapsed(3 * 3600), "3 hours ago");
    assert_eq!(elapsed(2 * 86400), "2 days ago");
    assert_eq!(elapsed(60 * 86400), "2 months ago");
    assert_eq!(elapsed(800 * 86400), "2 years ago");
}

#[test]
fn steps_through_paths_and_wraps_around() {
    let paths = [Path::new("/a"), Path::new("/b"), Path::new("/c")];

    assert_eq!(step(&paths, None, 1), Some(Path::new("/a")));
    assert_eq!(step(&paths, None, -1), Some(Path::new("/c")));
    assert_eq!(
        step(&paths, Some(Path::new("/c")), 1),
        Some(Path::new("/a"))
    );
    assert_eq!(
        step(&paths, Some(Path::new("/a")), -1),
        Some(Path::new("/c"))
    );
    assert_eq!(step(&[], None, 1), None);
}

#[test]
fn moving_a_repository_takes_the_slot_of_the_one_it_is_dropped_on() {
    let mut settings = Settings::default();
    settings.create_group("g");
    for name in ["a", "b", "c", "d"] {
        settings.add_to_group("g", Path::new(name));
    }
    let order = |settings: &Settings| -> Vec<String> {
        settings
            .group("g")
            .unwrap()
            .paths
            .iter()
            .map(|path| path.display().to_string())
            .collect()
    };
    assert!(settings.move_in_group("g", Path::new("d"), Path::new("a")));
    assert_eq!(order(&settings), ["d", "a", "b", "c"]);
    assert!(settings.move_in_group("g", Path::new("d"), Path::new("c")));
    assert_eq!(order(&settings), ["a", "b", "c", "d"]);
    assert!(!settings.move_in_group("g", Path::new("a"), Path::new("a")));
    assert!(!settings.move_in_group("g", Path::new("a"), Path::new("missing")));
    assert!(!settings.move_in_group("nope", Path::new("a"), Path::new("b")));
}

#[test]
fn shows_a_path_under_home_with_a_tilde_and_cuts_the_front_when_too_long() {
    let home = Path::new("/Users/me");
    assert_eq!(display_path(Path::new("/Users/me"), Some(home), 70), "~");
    assert_eq!(
        display_path(Path::new("/Users/me/work/a"), Some(home), 70),
        "~/work/a"
    );
    assert_eq!(display_path(Path::new("/srv/a"), Some(home), 70), "/srv/a");
    assert_eq!(display_path(Path::new("/srv/a"), None, 70), "/srv/a");
    assert_eq!(
        display_path(Path::new("/Users/me/work/naran"), Some(home), 8),
        "…k/naran"
    );
}
