#![allow(clippy::too_many_lines, reason = "a test reads from top to bottom")]

#[cfg(test)]
mod common;

use std::fs;

use naran_core::ops;
use naran_core::{BLOB_LIMIT, Error, FileInCommit, Page, Staging, worktree_file};

use common::{commit_file, compare, init, tag, write};

const MISSING: &str = "0123456789abcdef0123456789abcdef01234567";

fn invalid(error: &Error) -> bool {
    matches!(error, Error::InvalidArgument { .. })
}

#[test]
fn a_file_over_the_limit_or_a_folder_is_not_read() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path();
    fs::File::create(path.join("big.bin"))
        .unwrap()
        .set_len(BLOB_LIMIT + 1)
        .unwrap();
    fs::create_dir(path.join("folder")).unwrap();

    assert!(invalid(&worktree_file(path, "big.bin").unwrap_err()));
    assert!(invalid(&worktree_file(path, "folder").unwrap_err()));
}

#[test]
fn a_commit_that_does_not_exist_is_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "1\n", "첫째");

    assert!(naran_core::commit_detail(&path, MISSING).is_err());
    assert!(naran_core::commit_files(&path, MISSING).is_err());
}

#[test]
fn reading_outside_a_repository_is_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path();

    assert!(naran_core::compare_files(path, &compare("a", "b")).is_err());
    assert!(naran_core::file_history(path, "a.txt", 5).is_err());
    assert!(ops::branches(path).is_err());
    assert!(ops::changed_paths(path).is_err());
}

#[test]
fn the_changed_paths_are_the_modified_and_the_new_files() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "1\n", "첫째");
    write(&path, "a.txt", "2\n");
    write(&path, "new.txt", "1\n");

    let mut paths = ops::changed_paths(&path).unwrap();
    paths.sort();

    assert_eq!(paths, ["a.txt", "new.txt"]);
}

#[test]
fn discarding_nothing_is_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());

    assert!(ops::discard(&path, &[]).is_err());
    assert!(ops::discard_untracked(&path, &[]).is_err());
}

const BAD_NAME: &str = "--bad";
const BAD_HASH: &str = "not-a-hash";

fn all_failed(results: Vec<(&str, bool)>) {
    for (name, failed) in results {
        assert!(failed, "{name} should have failed");
    }
}

#[test]
fn every_operation_refuses_a_bad_name_hash_or_selection_before_running_git() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "1\n", "첫째");

    all_failed(vec![
        (
            "commit_detail",
            naran_core::commit_detail(&path, BAD_HASH).is_err(),
        ),
        (
            "commit_diff",
            naran_core::commit_diff(
                &path,
                &FileInCommit {
                    hash: BAD_HASH,
                    file: "a.txt",
                },
            )
            .is_err(),
        ),
        (
            "compare from",
            naran_core::compare_files(&path, &compare(BAD_NAME, "main")).is_err(),
        ),
        (
            "compare to",
            naran_core::compare_files(&path, &compare("main", BAD_NAME)).is_err(),
        ),
        (
            "compare diff from",
            naran_core::compare_diff(&path, &compare(BAD_NAME, "main"), "a").is_err(),
        ),
        (
            "compare diff to",
            naran_core::compare_diff(&path, &compare("main", BAD_NAME), "a").is_err(),
        ),
        (
            "blame",
            naran_core::blame(&path, "a.txt", Some(BAD_HASH)).is_err(),
        ),
        (
            "blob",
            naran_core::blob(&path, &naran_core::Side::Commit(BAD_HASH.into()), "a").is_err(),
        ),
        ("switch", ops::switch(&path, BAD_NAME).is_err()),
        (
            "switch_detached",
            ops::switch_detached(&path, BAD_HASH).is_err(),
        ),
        (
            "create_branch name",
            ops::create_branch(&path, BAD_NAME, None).is_err(),
        ),
        (
            "create_branch start",
            ops::create_branch(&path, "ok", Some(BAD_NAME)).is_err(),
        ),
        (
            "create_and_switch",
            ops::create_and_switch(&path, BAD_NAME, None).is_err(),
        ),
        (
            "delete_branch",
            ops::delete_branch(&path, BAD_NAME).is_err(),
        ),
        (
            "force_delete_branch",
            ops::force_delete_branch(&path, BAD_NAME).is_err(),
        ),
        (
            "unmerged_count",
            ops::unmerged_count(&path, BAD_NAME).is_err(),
        ),
        (
            "rename old",
            ops::rename_branch(&path, BAD_NAME, "new").is_err(),
        ),
        (
            "rename new",
            ops::rename_branch(&path, "old", BAD_NAME).is_err(),
        ),
        ("merge", ops::merge(&path, BAD_NAME).is_err()),
        ("rebase", ops::rebase(&path, BAD_NAME).is_err()),
        ("cherry_pick", ops::cherry_pick(&path, BAD_HASH).is_err()),
        ("revert", ops::revert(&path, BAD_HASH).is_err()),
        ("stash_apply", ops::stash_apply(&path, "bad").is_err()),
        ("stash_drop", ops::stash_drop(&path, "bad").is_err()),
        (
            "tag_create name",
            ops::tag_create(&path, &tag(BAD_NAME, None, None)).is_err(),
        ),
        (
            "tag_create target",
            ops::tag_create(&path, &tag("v1", Some(BAD_HASH), None)).is_err(),
        ),
        ("tag_delete", ops::tag_delete(&path, BAD_NAME).is_err()),
        ("unstage", ops::unstage(&path, &[]).is_err()),
    ]);
}

#[test]
fn unstaging_nothing_before_the_first_commit_is_an_error_too() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());

    assert!(ops::unstage(&path, &[]).is_err());
}

#[test]
fn every_operation_fails_when_git_does_outside_a_repository() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path();
    let files = ["a.txt".to_string()];

    all_failed(vec![
        ("summarize", naran_core::summarize(path).is_err()),
        (
            "history_page",
            naran_core::history_page(path, Page { skip: 0, limit: 5 }, naran_core::Refs::ALL)
                .is_err(),
        ),
        (
            "search",
            naran_core::search(
                path,
                &naran_core::Search {
                    kind: naran_core::SearchKind::Message,
                    query: "x",
                    limit: 5,
                },
                naran_core::Refs::ALL,
            )
            .is_err(),
        ),
        ("blame", naran_core::blame(path, "a.txt", None).is_err()),
        ("stage", ops::stage(path, &files).is_err()),
        ("unstage", ops::unstage(path, &files).is_err()),
        (
            "stage_lines",
            ops::stage_lines(
                path,
                &ops::LineSelection {
                    file: "a.txt",
                    staging: Staging::Unstaged,
                    lines: &[1],
                },
            )
            .is_err(),
        ),
        ("last_message", ops::last_message(path).is_err()),
        ("pull_safe", ops::pull_safe(path).is_err()),
        ("unmerged_count", ops::unmerged_count(path, "main").is_err()),
        ("abort_operation", ops::abort_operation(path).is_err()),
        ("continue_operation", ops::continue_operation(path).is_err()),
        ("resolve", ops::resolve(path, "a.txt", None).is_err()),
        (
            "resolve side",
            ops::resolve(path, "a.txt", Some(ops::Side::Theirs)).is_err(),
        ),
        ("stash_list", ops::stash_list(path).is_err()),
        ("tags", ops::tags(path).is_err()),
        ("submodules", ops::submodules(path).is_err()),
    ]);
}

#[test]
fn a_blob_over_the_limit_is_refused_even_when_git_has_it() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "small.txt", "1\n", "첫째");
    fs::File::create(path.join("big.bin"))
        .unwrap()
        .set_len(BLOB_LIMIT + 1)
        .unwrap();
    common::git(&path, &["add", "big.bin"]);
    common::git(&path, &["commit", "-q", "-m", "big"]);

    let error = naran_core::blob(&path, &naran_core::Side::Head, "big.bin").unwrap_err();

    assert!(invalid(&error));
}

#[test]
fn a_safe_pull_fails_when_the_remote_is_gone() {
    let directory = tempfile::tempdir().unwrap();
    let (remote, first, second) = common::with_remote(directory.path());
    commit_file(&first, "b.txt", "2\n", "둘째");
    common::git(&first, &["push", "-q"]);
    common::git(&second, &["fetch", "-q"]);
    fs::remove_dir_all(&remote).unwrap();

    assert!(ops::pull_safe(&second).is_err());
}
