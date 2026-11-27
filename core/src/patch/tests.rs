use super::*;

#[test]
fn a_header_that_is_not_a_hunk_header_is_refused() {
    assert_eq!(
        parse_header("@@ -3,2 +5 @@ fn x"),
        Some(Start { old: 3, new: 5 })
    );
    for broken in ["@@", "@@ -3", "@@ 3 +4", "@@ -x +4", "@@ -3 4", "@@ -3 +y"] {
        assert_eq!(parse_header(broken), None, "{broken}");
    }
}

const DIFF: &str = "diff --git a/f b/f\n--- a/f\n+++ b/f\n\
    @@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n\
    @@ -20,3 +20,4 @@\n x\n+y\n+z\n w\n";

fn index_of(needle: &str) -> usize {
    DIFF.split('\n').position(|line| line == needle).unwrap()
}

#[test]
fn selects_a_whole_hunk() {
    let selected = [index_of("-b"), index_of("+B")];

    let patch = build(DIFF, &selected, Direction::Forward).unwrap();

    assert!(patch.starts_with("diff --git a/f b/f\n--- a/f\n+++ b/f\n@@ -1,3 +1,3 @@\n"));
    assert!(patch.contains("-b\n+B\n"));
    assert!(!patch.contains("+y"));
}

#[test]
fn unselected_changes_keep_their_original_state() {
    let forward = build(DIFF, &[index_of("-b")], Direction::Forward).unwrap();
    let reverse = build(DIFF, &[index_of("-b")], Direction::Reverse).unwrap();

    assert!(forward.contains("@@ -1,3 +1,2 @@\n a\n-b\n c\n"));
    assert!(reverse.contains("@@ -1,4 +1,3 @@\n a\n-b\n B\n c\n"));
}

#[test]
fn selecting_one_added_line_turns_the_other_into_nothing_or_context() {
    let forward = build(DIFF, &[index_of("+y")], Direction::Forward).unwrap();
    let reverse = build(DIFF, &[index_of("+y")], Direction::Reverse).unwrap();

    assert!(forward.contains("@@ -20,2 +20,3 @@\n x\n+y\n w\n"));
    assert!(!forward.contains("+z"));
    assert!(reverse.contains("@@ -20,3 +20,4 @@\n x\n+y\n z\n w\n"));
}

#[test]
fn keeps_no_newline_markers_only_after_kept_lines() {
    let diff = "--- a/f\n+++ b/f\n@@ -1 +1 @@\n-a\n\\ No newline at end of file\n+b\n\\ No newline at end of file\n";
    let all: Vec<usize> = (0..diff.split('\n').count()).collect();

    let patch = build(diff, &all, Direction::Forward).unwrap();
    assert_eq!(patch.matches("\\ No newline").count(), 2);

    let only_removed = build(diff, &[3], Direction::Forward).unwrap();
    assert_eq!(only_removed.matches("\\ No newline").count(), 1);
}

#[test]
fn a_kept_last_line_gets_its_newline_back_when_a_line_is_added_after_it() {
    let diff =
        "--- a/f\n+++ b/f\n@@ -1 +1,2 @@\n-alpha\n\\ No newline at end of file\n+changed\n+more\n";

    let patch = build(diff, &[6], Direction::Forward).unwrap();

    let hunk = "@@ -1,1 +1,2 @@\n-alpha\n\\ No newline at end of file\n+alpha\n+more\n";
    assert!(patch.ends_with(hunk), "{patch}");
}

#[test]
fn a_kept_last_line_with_nothing_added_after_it_stays_context() {
    let diff = "--- a/f\n+++ b/f\n@@ -1,2 +1,2 @@\n-x\n+X\n-alpha\n\\ No newline at end of file\n";

    let patch = build(diff, &[3, 4], Direction::Forward).unwrap();

    assert!(
        patch.ends_with("-x\n+X\n alpha\n\\ No newline at end of file\n"),
        "{patch}"
    );
}

#[test]
fn returns_none_without_a_selection_or_hunks() {
    assert_eq!(build(DIFF, &[], Direction::Forward), None);
    assert_eq!(build("no hunks here", &[1], Direction::Forward), None);
    assert_eq!(
        build("@@ nonsense @@\n+a\n", &[1], Direction::Forward),
        None
    );
}

#[test]
fn a_renamed_file_is_named_by_its_new_path_alone_even_when_git_quotes_it() {
    let plain = "diff --git a/o b/n\nsimilarity index 90%\nrename from o\nrename to n\n--- a/o\n+++ b/n\n@@ -1 +1 @@\n-a\n+b\n";
    let quoted = plain.replace("rename to n", "rename to \"t\\tn\"");
    let index = |diff: &str| diff.split('\n').position(|line| line == "+b").unwrap();

    let patch = build(plain, &[index(plain)], Direction::Forward).unwrap();
    let other = build(&quoted, &[index(&quoted)], Direction::Forward).unwrap();

    assert!(patch.starts_with("diff --git a/n b/n\n--- a/n\n+++ b/n\n@@"));
    assert!(
        other.starts_with(
            "diff --git \"a/t\\tn\" \"b/t\\tn\"\n--- \"a/t\\tn\"\n+++ \"b/t\\tn\"\n@@"
        )
    );
}
