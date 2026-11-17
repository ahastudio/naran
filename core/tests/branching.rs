#![allow(clippy::too_many_lines, reason = "a test reads from top to bottom")]

#[cfg(test)]
mod common;

use naran_core::ops::{self, Side};
use naran_core::{Error, Operation, summarize};

use common::{commit_file, git, init, tag, write};

#[test]
fn manages_branches_and_merges() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "one\n", "첫째");

    ops::create_and_switch(&path, "feature", None).unwrap();
    commit_file(&path, "b.txt", "b\n", "기능");
    ops::switch(&path, "main").unwrap();
    assert_eq!(ops::unmerged_count(&path, "feature").unwrap(), 1);
    ops::create_branch(&path, "other", Some("main")).unwrap();

    ops::merge(&path, "feature").unwrap();
    ops::delete_branch(&path, "feature").unwrap();
    ops::rename_branch(&path, "other", "renamed").unwrap();

    let branches = ops::branches(&path).unwrap();
    let list: Vec<_> = branches.iter().map(|branch| branch.name.as_str()).collect();
    assert_eq!(list, ["main", "renamed"]);
    assert!(branches[0].current);

    // An unmerged branch is deleted only when forced.
    ops::switch(&path, "renamed").unwrap();
    commit_file(&path, "c.txt", "c\n", "미병합");
    ops::switch(&path, "main").unwrap();
    assert!(ops::delete_branch(&path, "renamed").is_err());
    ops::force_delete_branch(&path, "renamed").unwrap();

    ops::switch_detached(&path, git(&path, &["rev-parse", "HEAD"]).trim()).unwrap();
    assert_eq!(summarize(&path).unwrap().status.branch, None);
    assert!(ops::switch(&path, "--orphan").is_err());
}

#[test]
fn resolves_a_merge_conflict() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "base\n", "base");
    git(&path, &["switch", "-q", "-c", "other"]);
    commit_file(&path, "a.txt", "other\n", "other");
    git(&path, &["switch", "-q", "main"]);
    commit_file(&path, "a.txt", "main\n", "main");

    let error = ops::merge(&path, "other").unwrap_err();
    assert!(matches!(error, Error::Command { .. }), "{error}");
    assert_eq!(summarize(&path).unwrap().operation, Some(Operation::Merge));
    assert!(naran_core::changes(&path).unwrap()[0].conflicted);

    ops::abort_operation(&path).unwrap();
    assert_eq!(summarize(&path).unwrap().operation, None);
    assert!(ops::abort_operation(&path).is_err());

    let _ = ops::merge(&path, "other");
    ops::resolve(&path, "a.txt", Some(Side::Theirs)).unwrap();
    assert_eq!(
        std::fs::read_to_string(path.join("a.txt")).unwrap(),
        "other\n"
    );
    ops::continue_operation(&path).unwrap();
    assert_eq!(summarize(&path).unwrap().operation, None);
    assert_eq!(
        naran_core::history(&path, 10, naran_core::Refs::ALL)
            .unwrap()
            .commits[0]
            .parents
            .len(),
        2
    );
}

#[test]
fn rebases_cherry_picks_and_reverts() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "base\n", "base");
    git(&path, &["switch", "-q", "-c", "feature"]);
    commit_file(&path, "f.txt", "f\n", "feature");
    let feature = git(&path, &["rev-parse", "HEAD"]).trim().to_owned();
    git(&path, &["switch", "-q", "main"]);
    commit_file(&path, "m.txt", "m\n", "main");

    ops::cherry_pick(&path, &feature).unwrap();
    assert!(path.join("f.txt").exists());
    ops::revert(&path, git(&path, &["rev-parse", "HEAD"]).trim()).unwrap();
    assert!(!path.join("f.txt").exists());

    ops::switch(&path, "feature").unwrap();
    ops::rebase(&path, "main").unwrap();
    let history = naran_core::history(&path, 10, naran_core::Refs::ALL).unwrap();
    assert!(
        history
            .commits
            .iter()
            .any(|commit| commit.subject == "main")
    );

    // A conflicting rebase can be continued or aborted.
    commit_file(&path, "a.txt", "feature side\n", "feature a");
    git(&path, &["switch", "-q", "main"]);
    commit_file(&path, "a.txt", "main side\n", "main a");
    git(&path, &["switch", "-q", "feature"]);
    assert!(ops::rebase(&path, "main").is_err());
    assert_eq!(summarize(&path).unwrap().operation, Some(Operation::Rebase));
    ops::resolve(&path, "a.txt", None).unwrap();
    ops::continue_operation(&path).unwrap();
    assert_eq!(summarize(&path).unwrap().operation, None);

    assert!(ops::cherry_pick(&path, "--abort").is_err());
}

#[test]
fn stashes_and_tags() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "one\n", "첫째");
    write(&path, "a.txt", "two\n");
    write(&path, "u.txt", "untracked\n");

    ops::stash_push(&path, "작업 중", ops::Untracked::Include).unwrap();
    assert_eq!(
        naran_core::changes(&path).unwrap(),
        [] as [naran_core::Change; 0]
    );
    let stashes = ops::stash_list(&path).unwrap();
    assert_eq!(stashes[0].name, "stash@{0}");
    assert!(stashes[0].message.contains("작업 중"));

    ops::stash_apply(&path, "stash@{0}").unwrap();
    assert_eq!(naran_core::changes(&path).unwrap().len(), 2);
    ops::stash_drop(&path, "stash@{0}").unwrap();
    assert_eq!(ops::stash_list(&path).unwrap(), [] as [ops::Stash; 0]);
    assert!(ops::stash_apply(&path, "--all").is_err());

    let hash = git(&path, &["rev-parse", "HEAD"]).trim().to_owned();
    ops::tag_create(&path, &tag("v1", None, None)).unwrap();
    ops::tag_create(&path, &tag("v2", Some(&hash), Some("주석 태그"))).unwrap();
    assert_eq!(ops::tags(&path).unwrap().len(), 2);
    ops::tag_delete(&path, "v1").unwrap();
    assert_eq!(ops::tags(&path).unwrap(), ["v2"]);
    assert!(ops::tag_create(&path, &tag("-bad", None, None)).is_err());
    assert!(ops::tag_create(&path, &tag("v3", Some("--x"), None)).is_err());
}
