#![allow(clippy::too_many_lines, reason = "a test reads from top to bottom")]

#[cfg(test)]
mod common;

use naran_core::ops;
use naran_core::{Error, Staging};

use common::{commit_file, git, init, write};

#[test]
fn a_staged_rename_lists_the_new_path_and_the_old_one() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "old name.txt", "1\n", "첫 커밋");
    git(&path, &["mv", "old name.txt", "new name.txt"]);
    write(&path, "loose.txt", "2\n");

    let changes = naran_core::changes(&path).unwrap();

    let renamed = changes.iter().find(|change| change.index == 'R').unwrap();
    assert_eq!(renamed.path, "new name.txt");
    assert_eq!(renamed.original.as_deref(), Some("old name.txt"));
    let loose = changes.iter().find(|change| change.path == "loose.txt");
    assert_eq!(loose.unwrap().original, None);
}

#[test]
fn unstaging_a_renamed_file_brings_back_the_old_path() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "old name.txt", "1\n", "첫 커밋");
    std::fs::create_dir(path.join("dir")).unwrap();
    git(&path, &["mv", "old name.txt", "dir/new name.txt"]);

    ops::unstage(&path, &["dir/new name.txt".into()]).unwrap();

    let changes = naran_core::changes(&path).unwrap();
    let state = |name: &str| {
        let change = changes.iter().find(|change| change.path == name).unwrap();
        (change.index, change.worktree)
    };
    assert_eq!(state("old name.txt"), ('.', 'D'), "{changes:?}");
    assert_eq!(state("dir/new name.txt"), ('?', '?'), "{changes:?}");
}

#[test]
fn stages_unstages_and_commits() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    write(&path, "새 파일.txt", "1\n");

    ops::stage(&path, &["새 파일.txt".into()]).unwrap();
    assert_eq!(naran_core::changes(&path).unwrap()[0].index, 'A');

    // Unstaging works even before the first commit.
    ops::unstage(&path, &["새 파일.txt".into()]).unwrap();
    assert_eq!(naran_core::changes(&path).unwrap()[0].index, '?');

    ops::stage(&path, &["새 파일.txt".into()]).unwrap();
    ops::commit(&path, "첫 커밋\n\n본문").unwrap();
    assert_eq!(ops::last_message(&path).unwrap(), "첫 커밋\n\n본문");

    write(&path, "새 파일.txt", "2\n");
    ops::stage(&path, &["새 파일.txt".into()]).unwrap();
    ops::amend(&path, "").unwrap();
    assert_eq!(ops::last_message(&path).unwrap(), "첫 커밋\n\n본문");
    ops::amend(&path, "고친 메시지").unwrap();
    assert_eq!(ops::last_message(&path).unwrap(), "고친 메시지");
    assert_eq!(
        naran_core::history(&path, 10, naran_core::Refs::ALL)
            .unwrap()
            .commits
            .len(),
        1
    );

    assert!(ops::commit(&path, "  ").is_err());
    assert!(ops::stage(&path, &[]).is_err());
}

#[test]
fn discards_tracked_and_untracked_changes() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "a.txt", "one\n", "첫째");
    write(&path, "a.txt", "changed\n");
    git(&path, &["add", "a.txt"]);
    write(&path, "junk.txt", "x\n");

    ops::discard(&path, &["a.txt".into()]).unwrap();
    ops::discard_untracked(&path, &["junk.txt".into()]).unwrap();

    assert_eq!(
        naran_core::changes(&path).unwrap(),
        [] as [naran_core::Change; 0]
    );
    assert_eq!(
        std::fs::read_to_string(path.join("a.txt")).unwrap(),
        "one\n"
    );
}

fn numbered(change: &[(usize, &str)]) -> String {
    (1..=12)
        .map(|number| {
            change
                .iter()
                .find(|(line, _)| *line == number)
                .map_or_else(|| format!("line{number}"), |(_, text)| (*text).to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn indexes(diff: &str, wanted: &[&str]) -> Vec<usize> {
    diff.split('\n')
        .enumerate()
        .filter(|(_, line)| wanted.contains(line))
        .map(|(index, _)| index)
        .collect()
}

/// Lines of `f.txt` picked in one of its diffs.
fn selection(staging: Staging, lines: &[usize]) -> ops::LineSelection<'_> {
    ops::LineSelection {
        file: "f.txt",
        staging,
        lines,
    }
}

#[test]
fn stages_and_unstages_selected_hunks_and_lines() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "f.txt", &numbered(&[]), "base");
    write(
        &path,
        "f.txt",
        &numbered(&[(1, "FIRST"), (2, "SECOND"), (12, "LAST")]),
    );
    let diff = naran_core::file_diff(&path, "f.txt", Staging::Unstaged).unwrap();

    // Stage a single hunk (the change in the first two lines).
    let first_hunk = indexes(&diff, &["-line1", "+FIRST", "-line2", "+SECOND"]);
    ops::stage_lines(&path, &selection(Staging::Unstaged, &first_hunk)).unwrap();
    let cached = naran_core::file_diff(&path, "f.txt", Staging::Staged).unwrap();
    assert!(cached.contains("+FIRST") && cached.contains("+SECOND"));
    assert!(!cached.contains("+LAST"));
    let rest = naran_core::file_diff(&path, "f.txt", Staging::Unstaged).unwrap();
    assert!(rest.contains("+LAST") && !rest.contains("+FIRST"));

    // Unstage only one pair of lines from what is staged.
    let unstage = indexes(&cached, &["-line2", "+SECOND"]);
    ops::stage_lines(&path, &selection(Staging::Staged, &unstage)).unwrap();
    let cached = naran_core::file_diff(&path, "f.txt", Staging::Staged).unwrap();
    assert!(cached.contains("+FIRST") && !cached.contains("+SECOND"));

    // Stage a single line inside one hunk.
    let worktree = naran_core::file_diff(&path, "f.txt", Staging::Unstaged).unwrap();
    let only_last = indexes(&worktree, &["+LAST"]);
    ops::stage_lines(&path, &selection(Staging::Unstaged, &only_last)).unwrap();
    let cached = naran_core::file_diff(&path, "f.txt", Staging::Staged).unwrap();
    assert!(cached.contains("+LAST"));

    assert!(matches!(
        ops::stage_lines(&path, &selection(Staging::Unstaged, &[0])),
        Err(Error::InvalidArgument { .. })
    ));
}

#[test]
fn a_staged_rename_shows_only_what_changed_and_its_lines_unstage_without_undoing_the_rename() {
    let directory = tempfile::tempdir().unwrap();
    let path = init(directory.path());
    commit_file(&path, "old.txt", &numbered(&[]), "base");
    git(&path, &["mv", "old.txt", "한글.txt"]);
    write(&path, "한글.txt", &numbered(&[(2, "CHANGED")]));
    git(&path, &["add", "한글.txt"]);

    let diff = naran_core::file_diff(&path, "한글.txt", Staging::Staged).unwrap();

    assert!(diff.contains("+CHANGED") && diff.contains("-line2"));
    assert!(
        !diff.contains("+line1"),
        "unchanged lines are not added lines"
    );
    let lines = indexes(&diff, &["-line2", "+CHANGED"]);
    let picked = ops::LineSelection {
        file: "한글.txt",
        staging: Staging::Staged,
        lines: &lines,
    };
    ops::stage_lines(&path, &picked).unwrap();
    let status = git(&path, &["-c", "core.quotePath=false", "status", "--short"]);
    assert_eq!(status.trim(), "RM old.txt -> 한글.txt");

    let plain = tempfile::tempdir().unwrap();
    let unreadable = naran_core::file_diff(plain.path(), "a.txt", Staging::Staged);
    assert!(matches!(unreadable, Err(Error::Command { .. })));
}
