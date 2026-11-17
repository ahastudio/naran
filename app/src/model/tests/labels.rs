use naran_core::{Change, Staging};

use super::*;

fn renamed(path: &str, original: Option<&str>) -> Change {
    Change {
        path: path.into(),
        original: original.map(Into::into),
        index: 'R',
        worktree: '.',
        conflicted: false,
    }
}

#[test]
fn a_renamed_file_is_labeled_with_its_new_path_then_the_old_one() {
    assert_eq!(
        change_label(
            &renamed("docs/new name.md", Some("old name.md")),
            Staging::Staged
        ),
        "docs/new name.md <- old name.md"
    );
    let mut copy = renamed("copy.md", Some("source.md"));
    copy.index = 'C';
    assert_eq!(change_label(&copy, Staging::Staged), "copy.md <- source.md");
}

#[test]
fn a_file_that_was_not_renamed_is_labeled_with_its_path_only() {
    assert_eq!(
        change_label(&renamed("a.txt", None), Staging::Staged),
        "a.txt"
    );
}

#[test]
fn a_rename_that_is_staged_is_not_shown_as_one_in_the_unstaged_list() {
    let mut change = renamed("new.md", Some("old.md"));
    change.worktree = 'M';
    assert_eq!(change_label(&change, Staging::Unstaged), "new.md");
    assert_eq!(change_label(&change, Staging::Staged), "new.md <- old.md");
}
