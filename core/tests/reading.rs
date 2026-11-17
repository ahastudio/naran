#![allow(clippy::too_many_lines, reason = "a test reads from top to bottom")]

#[cfg(test)]
mod common;

use naran_core::SearchKind::{Author, File, Message};
use naran_core::ops;
use naran_core::{Error, Page, Search, summarize};

use common::{commit_file, compare, git, init, write};

fn names(files: &[naran_core::CommitFile]) -> Vec<&str> {
    files.iter().map(|file| file.path.as_str()).collect()
}

#[test]
fn blames_searches_and_compares() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "one\ntwo\n", "첫째 커밋");
    git(&path, &["switch", "-q", "-c", "topic"]);
    write(&path, "a.txt", "one\ntwo\nthree\n");
    git(&path, &["add", "."]);
    git(
        &path,
        &[
            "-c",
            "user.name=Ada",
            "commit",
            "-q",
            "-m",
            "셋째 줄 추가",
            "--author=Ada <ada@example.com>",
        ],
    );
    git(&path, &["mv", "a.txt", "renamed.txt"]);
    git(&path, &["commit", "-q", "-m", "이름 변경"]);

    let blame = naran_core::blame(&path, "renamed.txt", None).unwrap();
    assert_eq!(blame.len(), 3);
    assert_eq!(
        (blame[0].author.as_str(), blame[2].author.as_str()),
        ("Naran", "Ada")
    );

    let history = naran_core::file_history(&path, "renamed.txt", 10).unwrap();
    assert_eq!(history.len(), 3);

    let found = |kind, query| {
        naran_core::search(
            &path,
            &Search {
                kind,
                query,
                limit: 50,
            },
            naran_core::Refs::ALL,
        )
        .unwrap()
        .len()
    };
    assert_eq!(found(Message, "셋째"), 1);
    assert_eq!(found(Message, "없는 말"), 0);
    assert_eq!(found(Author, "ada"), 1);
    assert_eq!(found(File, "renamed.txt"), 1);

    let files = naran_core::compare_files(&path, &compare("main", "topic")).unwrap();
    assert_eq!(names(&files), ["a.txt", "renamed.txt"]);
    let diff = naran_core::compare_diff(&path, &compare("main", "topic"), "renamed.txt").unwrap();
    assert!(diff.contains("+three"));
    assert!(naran_core::compare_files(&path, &compare("--x", "topic")).is_err());
}

#[test]
fn runs_in_parallel_across_repositories() {
    let directory = tempfile::tempdir().unwrap();
    let paths: Vec<_> = (0..6)
        .map(|number| {
            let path = init(&directory.path().join(format!("r{number}")));
            commit_file(&path, "a.txt", "1\n", "x");
            path
        })
        .collect();

    let results = naran_core::batch::parallel(&paths, 3, |path| summarize(path).map(|s| s.name));

    let found: Vec<_> = results.into_iter().map(|(_, name)| name.unwrap()).collect();
    assert_eq!(found, ["r0", "r1", "r2", "r3", "r4", "r5"]);
}

#[test]
fn reports_an_error_for_a_missing_repository() {
    let error =
        ops::stash_apply(std::path::Path::new("/definitely/not/here"), "stash@{0}").unwrap_err();

    assert!(matches!(error, Error::Command { .. } | Error::Spawn { .. }));
}

#[test]
fn reads_files_as_they_are_on_each_side() {
    use naran_core::{Side, blob, worktree_file};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path();
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init", "-q"]);
    run(&["config", "user.email", "a@b.c"]);
    run(&["config", "user.name", "n"]);
    let first: Vec<u8> = (0..=255u8).collect(); // bytes that are not valid text
    std::fs::write(path.join("pic.png"), &first).unwrap();
    run(&["add", "pic.png"]);
    run(&["commit", "-q", "-m", "first"]);
    let second = vec![0u8, 255, 254, 1, 2, 3];
    std::fs::write(path.join("pic.png"), &second).unwrap();
    run(&["add", "pic.png"]);
    let third = vec![9u8, 8, 7];
    std::fs::write(path.join("pic.png"), &third).unwrap();

    assert_eq!(
        blob(path, &Side::Head, "pic.png").unwrap(),
        Some(first.clone())
    );
    assert_eq!(blob(path, &Side::Index, "pic.png").unwrap(), Some(second));
    assert_eq!(worktree_file(path, "pic.png").unwrap(), Some(third));
    assert_eq!(blob(path, &Side::Head, "missing.png").unwrap(), None);
    assert_eq!(worktree_file(path, "missing.png").unwrap(), None);
    let hash = String::from_utf8(
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let hash = hash.trim().to_string();
    assert_eq!(
        blob(path, &Side::Commit(hash.clone()), "pic.png").unwrap(),
        Some(first)
    );
    assert_eq!(
        blob(path, &Side::ParentOf(hash), "pic.png").unwrap(),
        None,
        "a first commit has no parent"
    );
    assert!(blob(path, &Side::Commit("--help".into()), "pic.png").is_err());
}

#[test]
fn history_pages_join_into_the_full_history() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path();
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init", "-q", "-b", "main"]);
    run(&["config", "user.email", "a@b.c"]);
    run(&["config", "user.name", "n"]);
    run(&["commit", "-q", "--allow-empty", "-m", "c1"]);
    run(&["checkout", "-q", "-b", "topic"]);
    run(&["commit", "-q", "--allow-empty", "-m", "t1"]);
    run(&["checkout", "-q", "main"]);
    run(&["commit", "-q", "--allow-empty", "-m", "c2"]);
    run(&["merge", "-q", "--no-ff", "-m", "merge", "topic"]);
    run(&["commit", "-q", "--allow-empty", "-m", "c3"]);

    let all = naran_core::history(path, 100, naran_core::Refs::ALL).unwrap();
    assert_eq!(all.commits.len(), 5);
    let mut paged = Vec::new();
    for skip in [0, 2, 4] {
        paged.extend(
            naran_core::history_page(path, Page { skip, limit: 2 }, naran_core::Refs::ALL).unwrap(),
        );
    }
    assert_eq!(paged, all.commits, "pages in order are the same commits");
    assert_eq!(
        naran_core::history_page(path, Page { skip: 5, limit: 2 }, naran_core::Refs::ALL).unwrap(),
        [] as [naran_core::log::Commit; 0]
    );
    // The graph of the joined list equals the graph read in one piece.
    assert_eq!(naran_core::graph::layout(&paged), all.rows);
}

#[test]
fn history_hides_refs_git_keeps_for_itself_unless_asked() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "1\n", "c1");
    commit_file(&path, "a.txt", "2\n", "c2");
    // What `git filter-branch` leaves behind: a backup of the branch as it was.
    git(
        &path,
        &["update-ref", "refs/original/refs/heads/main", "HEAD"],
    );
    git(&path, &["reset", "-q", "--hard", "HEAD~1"]);

    let visible = naran_core::history(&path, 10, naran_core::Refs::VISIBLE).unwrap();
    let all = naran_core::history(&path, 10, naran_core::Refs::ALL).unwrap();
    assert_eq!(visible.commits.len(), 1);
    assert_eq!(all.commits.len(), 2);
    let page = naran_core::history_page(
        &path,
        Page { skip: 0, limit: 10 },
        naran_core::Refs::VISIBLE,
    )
    .unwrap();
    assert_eq!(page.len(), 1);
}

#[test]
fn remote_branches_can_be_left_out_of_the_history() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().to_path_buf();
    git(&path, &["init", "-q", "-b", "main"]);
    commit_file(&path, "a.txt", "1\n", "c1");
    commit_file(&path, "a.txt", "2\n", "c2");
    git(&path, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&path, &["reset", "-q", "--hard", "HEAD~1"]);

    for everything in [false, true] {
        let with = naran_core::Refs {
            everything,
            remotes: true,
        };
        let without = naran_core::Refs {
            everything,
            remotes: false,
        };
        assert_eq!(
            naran_core::history(&path, 10, with).unwrap().commits.len(),
            2
        );
        assert_eq!(
            naran_core::history(&path, 10, without)
                .unwrap()
                .commits
                .len(),
            1
        );
    }
}

#[test]
fn refs_read_the_older_saved_form() {
    let old: naran_core::Refs = serde_json::from_str("\"All\"").unwrap();
    assert_eq!(old, naran_core::Refs::ALL);
    let old: naran_core::Refs = serde_json::from_str("\"Visible\"").unwrap();
    assert_eq!(old, naran_core::Refs::VISIBLE);
    let flags = naran_core::Refs {
        everything: true,
        remotes: false,
    };
    let again: naran_core::Refs =
        serde_json::from_str(&serde_json::to_string(&flags).unwrap()).unwrap();
    assert_eq!(again, flags);
}
