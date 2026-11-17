#![allow(clippy::too_many_lines, reason = "a test reads from top to bottom")]

use std::fs;
use std::process::Command;

use naran_core::{Error, FileInCommit, Operation, Staging, summarize};

#[cfg(test)]
mod common;

use common::{git, repository};

#[test]
fn counts_changes() {
    let directory = repository();
    let path = directory.path();
    fs::write(path.join("커밋됨.txt"), "1").unwrap();
    git(path, &["add", "."]);
    git(path, &["commit", "-q", "-m", "처음"]);
    fs::write(path.join("커밋됨.txt"), "2").unwrap();
    fs::write(path.join("staged.txt"), "1").unwrap();
    git(path, &["add", "staged.txt"]);
    fs::write(path.join("new file.txt"), "1").unwrap();

    let summary = summarize(path).unwrap();

    assert_eq!(summary.status.branch.as_deref(), Some("main"));
    assert!(summary.status.head.is_some());
    assert_eq!(summary.status.staged, 1);
    assert_eq!(summary.status.unstaged, 1);
    assert_eq!(summary.status.untracked, 1);
    assert_eq!(summary.operation, None);
}

#[test]
fn detects_merge_conflict() {
    let directory = repository();
    let path = directory.path();
    fs::write(path.join("a.txt"), "base").unwrap();
    git(path, &["add", "."]);
    git(path, &["commit", "-q", "-m", "base"]);
    git(path, &["switch", "-q", "-c", "other"]);
    fs::write(path.join("a.txt"), "other").unwrap();
    git(path, &["commit", "-q", "-am", "other"]);
    git(path, &["switch", "-q", "main"]);
    fs::write(path.join("a.txt"), "main").unwrap();
    git(path, &["commit", "-q", "-am", "main"]);
    let _ = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["merge", "other"])
        .output();

    let summary = summarize(path).unwrap();

    assert_eq!(summary.status.conflicted, 1);
    assert_eq!(summary.operation, Some(Operation::Merge));
}

#[test]
fn reports_non_repository() {
    let directory = tempfile::tempdir().unwrap();

    let error = summarize(directory.path()).unwrap_err();

    assert!(matches!(error, Error::Command { .. }));
}

#[test]
fn reads_history_changes_and_diffs() {
    let directory = repository();
    let path = directory.path();
    fs::write(path.join("a.txt"), "one\n").unwrap();
    git(path, &["add", "."]);
    git(path, &["commit", "-q", "-m", "첫째"]);
    fs::write(path.join("a.txt"), "one\ntwo\n").unwrap();
    git(path, &["commit", "-q", "-am", "둘째"]);
    fs::write(path.join("a.txt"), "one\ntwo\nthree\n").unwrap();

    let history = naran_core::history(path, 10, naran_core::Refs::ALL).unwrap();
    assert_eq!(history.commits.len(), 2);
    assert_eq!(history.commits[0].subject, "둘째");
    assert_eq!(history.rows.len(), 2);

    let hash = &history.commits[0].hash;
    let files = naran_core::commit_files(path, hash).unwrap();
    assert_eq!(files[0].path, "a.txt");
    assert_eq!(files[0].status, 'M');
    assert!(
        naran_core::commit_diff(
            path,
            &FileInCommit {
                hash,
                file: "a.txt"
            }
        )
        .unwrap()
        .contains("+two")
    );

    let changes = naran_core::changes(path).unwrap();
    assert_eq!(changes[0].worktree, 'M');
    assert!(
        naran_core::file_diff(path, "a.txt", Staging::Unstaged)
            .unwrap()
            .contains("+three")
    );
    assert_eq!(
        naran_core::file_diff(path, "a.txt", Staging::Staged).unwrap(),
        ""
    );
}

#[test]
fn history_of_empty_repository_is_empty() {
    let directory = repository();

    assert_eq!(
        naran_core::history(directory.path(), 10, naran_core::Refs::ALL)
            .unwrap()
            .commits,
        [] as [naran_core::log::Commit; 0]
    );
}

#[test]
fn rejects_option_like_hashes() {
    let directory = repository();

    let error = naran_core::commit_files(directory.path(), "--output=x").unwrap_err();

    assert!(matches!(error, Error::InvalidArgument { .. }));
    assert!(
        naran_core::commit_diff(
            directory.path(),
            &FileInCommit {
                hash: "",
                file: "a"
            }
        )
        .is_err()
    );
}

#[test]
fn every_file_in_a_new_folder_is_listed_and_counted() {
    let directory = repository();
    let path = directory.path();
    fs::create_dir_all(path.join("tools/five-lines/src")).unwrap();
    fs::write(path.join("tools/five-lines/Cargo.toml"), "").unwrap();
    fs::write(path.join("tools/five-lines/src/main.rs"), "").unwrap();

    let mut changed = naran_core::changes(path)
        .unwrap()
        .into_iter()
        .map(|change| change.path)
        .collect::<Vec<_>>();
    changed.sort();

    let files = [
        "tools/five-lines/Cargo.toml",
        "tools/five-lines/src/main.rs",
    ];
    assert_eq!(changed, files);
    assert_eq!(naran_core::ops::changed_paths(path).unwrap().len(), 2);
    assert_eq!(summarize(path).unwrap().status.untracked, 2);
}
