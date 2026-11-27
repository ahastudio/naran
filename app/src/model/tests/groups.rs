//! The order of the groups.

use super::*;

#[test]
fn moving_a_group_takes_the_slot_of_the_one_it_is_dropped_on() {
    let mut settings = Settings::default();
    for name in ["a", "b", "c", "d"] {
        settings.create_group(name);
    }
    let order = |settings: &Settings| -> Vec<String> {
        settings
            .groups
            .iter()
            .map(|group| group.name.clone())
            .collect()
    };
    assert!(settings.move_group("d", "a"));
    assert_eq!(order(&settings), ["d", "a", "b", "c"]);
    assert!(settings.move_group("d", "c"));
    assert_eq!(order(&settings), ["a", "b", "c", "d"]);
    assert!(!settings.move_group("a", "a"));
    assert!(!settings.move_group("a", "missing"));
    assert!(!settings.move_group("missing", "a"));
}

/// Groups `a` and `b`, each holding the given paths.
fn two_groups(a: &[&str], b: &[&str]) -> Settings {
    let mut settings = Settings::default();
    for (name, paths) in [("a", a), ("b", b)] {
        settings.create_group(name);
        for path in paths {
            settings.add_to_group(name, Path::new(path));
        }
    }
    settings
}

fn held(settings: &Settings, name: &str) -> Vec<String> {
    let group = settings.groups.iter().find(|group| group.name == name);
    let paths = group.map(|group| group.paths.clone()).unwrap_or_default();
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

#[test]
fn moving_a_repository_takes_it_out_of_its_group_and_puts_it_last_in_the_target() {
    let mut settings = two_groups(&["/x", "/y"], &["/z"]);
    assert!(settings.move_to_group("a", Path::new("/x"), "b"));
    assert_eq!(held(&settings, "a"), ["/y"]);
    assert_eq!(held(&settings, "b"), ["/z", "/x"]);
}

#[test]
fn moving_a_repository_to_a_group_that_holds_it_keeps_it_once() {
    let mut settings = two_groups(&["/x", "/y"], &["/x"]);
    assert!(settings.move_to_group("a", Path::new("/x"), "b"));
    assert_eq!(held(&settings, "a"), ["/y"]);
    assert_eq!(held(&settings, "b"), ["/x"]);
}

#[test]
fn dropping_a_repository_on_its_own_group_changes_nothing() {
    let mut settings = two_groups(&["/x", "/y"], &[]);
    assert!(!settings.move_to_group("a", Path::new("/x"), "a"));
    assert_eq!(held(&settings, "a"), ["/x", "/y"]);
}

#[test]
fn moving_a_repository_to_a_missing_group_changes_nothing() {
    let mut settings = two_groups(&["/x"], &[]);
    assert!(!settings.move_to_group("a", Path::new("/x"), "missing"));
    assert_eq!(held(&settings, "a"), ["/x"]);
}

#[test]
fn moving_a_repository_the_source_group_does_not_hold_changes_nothing() {
    let mut settings = two_groups(&["/x"], &[]);
    assert!(!settings.move_to_group("a", Path::new("/other"), "b"));
    assert!(!settings.move_to_group("missing", Path::new("/x"), "b"));
    assert_eq!(
        (held(&settings, "a"), held(&settings, "b")),
        (vec!["/x".to_string()], vec![])
    );
}
