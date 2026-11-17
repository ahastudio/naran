#![allow(clippy::too_many_lines, reason = "a test reads from top to bottom")]

#[cfg(test)]
mod common;

use naran_core::ops::{self, Outcome};
use naran_core::summarize;

use common::{commit_file, git, init, with_remote, write};

#[test]
fn fetches_pulls_and_pushes() {
    let directory = tempfile::tempdir().unwrap();
    let (_remote, first, second) = with_remote(directory.path());

    commit_file(&first, "b.txt", "b\n", "둘째");
    ops::push(&first).unwrap();

    // Fetching alone makes the repository show as behind.
    ops::fetch(&second).unwrap();
    assert_eq!(summarize(&second).unwrap().status.behind, 1);
    ops::pull(&second).unwrap();
    assert_eq!(summarize(&second).unwrap().status.behind, 0);
    assert!(second.join("b.txt").exists());

    commit_file(&second, "c.txt", "c\n", "셋째");
    assert_eq!(ops::branches(&second).unwrap()[0].ahead, 1);
    ops::push(&second).unwrap();
    assert_eq!(ops::branches(&second).unwrap()[0].ahead, 0);

    // With diverged history a plain push is rejected; a forced push after fetching succeeds.
    commit_file(&first, "d.txt", "d\n", "갈라짐");
    assert!(ops::push(&first).is_err());
    ops::fetch(&first).unwrap();
    ops::force_push(&first).unwrap();

    // A new branch without tracking is pushed to origin and starts tracking it.
    ops::create_and_switch(&first, "feature", None).unwrap();
    assert_eq!(summarize(&first).unwrap().status.upstream, None);
    ops::push(&first).unwrap();
    assert_eq!(
        summarize(&first).unwrap().status.upstream.as_deref(),
        Some("origin/feature")
    );
}

#[test]
fn pulls_only_when_safe() {
    let directory = tempfile::tempdir().unwrap();
    let (_remote, first, second) = with_remote(directory.path());
    let outcome = |path: &std::path::Path| match ops::pull_safe(path).unwrap() {
        Outcome::Skipped { reason } => reason,
        Outcome::Done => "done".to_owned(),
    };

    assert_eq!(outcome(&second), "not behind");

    commit_file(&first, "b.txt", "b\n", "둘째");
    ops::push(&first).unwrap();
    ops::fetch(&second).unwrap();

    write(&second, "a.txt", "dirty\n");
    assert_eq!(outcome(&second), "there are local changes");
    git(&second, &["checkout", "--", "a.txt"]);

    commit_file(&second, "local.txt", "x\n", "로컬");
    assert_eq!(outcome(&second), "local commits would need a merge");
    git(&second, &["reset", "-q", "--hard", "HEAD~1"]);

    assert_eq!(outcome(&second), "done");
    assert!(second.join("b.txt").exists());

    // Skipped when there is no remote.
    let lonely = init(&directory.path().join("lonely"));
    commit_file(&lonely, "a.txt", "1\n", "x");
    assert_eq!(outcome(&lonely), "no upstream branch");
}

#[test]
fn lists_submodules() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(&directory.path().join("main"));
    let sub = init(&directory.path().join("sub"));
    commit_file(&sub, "s.txt", "s\n", "sub");
    commit_file(&path, "a.txt", "1\n", "x");
    git(
        &path,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            sub.to_str().unwrap(),
            "libs/sub",
        ],
    );

    let modules = ops::submodules(&path).unwrap();

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].path, "libs/sub");
    assert_eq!(modules[0].state, "ok");
    assert_eq!(ops::submodules(&sub).unwrap(), [] as [ops::Submodule; 0]);
}
