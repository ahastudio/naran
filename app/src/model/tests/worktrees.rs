//! The worktrees of an entry.

use naran_core::Worktree;

use super::*;

#[test]
fn turns_worktrees_into_entries() {
    let linked = Summary {
        path: "/linked".into(),
        name: "linked".into(),
        status: Status::default(),
        operation: None,
        worktrees: Vec::new(),
    };
    let worktree = |path: &str, summary| Worktree {
        path: path.into(),
        locked: false,
        prunable: false,
        summary,
    };
    let missing = Error::InvalidArgument {
        message: "gone".into(),
    };
    let Entry::Ready(mut main) = ready("/main", Status::default()) else {
        unreachable!();
    };
    main.worktrees = vec![
        worktree("/linked", Ok(linked.clone())),
        worktree("/gone", Err(missing.clone())),
    ];

    let entry = Entry::Ready(main);
    let entries = entry.worktrees();
    assert_eq!(entry.worktree_count(), 2);

    assert_eq!(
        entries,
        [
            Entry::Ready(linked),
            Entry::Failed {
                path: "/gone".into(),
                error: missing
            }
        ]
    );
    assert_eq!(failed("/a").worktrees().len(), 0);
    assert_eq!(failed("/a").worktree_count(), 0);
}

#[test]
fn a_missing_worktree_folder_is_one_word_named_by_its_folder() {
    let missing = Entry::Failed {
        path: "/work/worktrees/feature".into(),
        error: Error::InvalidArgument {
            message: naran_core::MISSING_FOLDER.into(),
        },
    };

    assert_eq!(describe(&missing).0, "Missing");
    assert_eq!(missing.folder(), "feature");
    assert_eq!(failed("/").folder(), "/");
    assert_ne!(describe(&failed("/a")).0, "Missing");
}
