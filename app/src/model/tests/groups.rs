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
