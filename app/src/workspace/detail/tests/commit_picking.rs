//! Picking the files of a commit: what the picks do when the files are missing,
//! and the stacked diff read for them.

use super::*;
use crate::workspace::detail::commit::CommitKey;
use crate::workspace::detail::commit_picking::read_commit_stack;
use crate::workspace::detail::stack::{IMAGE_NOTE, StackRow};
use crate::workspace::tests::git;
use naran_core::CommitFile;

fn file(path: &str, status: char) -> CommitFile {
    CommitFile {
        path: path.into(),
        status,
    }
}

#[gpui::test]
fn picks_cannot_move_through_files_that_are_not_there(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx) = open(&fixture, cx);
    let mut inspecting = inspecting(&fixture);
    assert!(inspecting.extend(1).is_none(), "the files failed to read");
    assert!(inspecting.step_to(1).is_none());

    inspecting.files = Some(Ok(Vec::new()));
    assert!(inspecting.extend(1).is_none(), "a commit without files");
    assert!(inspecting.step_to(1).is_none());

    inspecting.files = Some(Ok(vec![file("a.txt", 'A'), file("b.txt", 'A')]));
    inspecting.file = None;
    assert_eq!(inspecting.extend(1), Some((0, "a.txt".into())));

    select(&workspace, &fixture.beta, cx);
    workspace.update(cx, |workspace, cx| {
        workspace.refresh_commit_stack(cx);
        workspace.click_commit_file("a.txt".into(), Modifiers::default(), cx);
        workspace.select_all_commit_files(cx);
        workspace.extend_commit_files(1, cx);
        workspace.selection.repository = None;
        workspace.refresh_stacked(cx);
    });
    workspace.read_with(cx, |workspace, _| {
        assert!(workspace.shown.stacked.is_none(), "no commit is picked");
    });
}

#[test]
fn the_stack_of_a_commit_has_its_files_images_and_the_count_left_out() {
    let fixture = fixture(&[]);
    std::fs::write(fixture.alpha.join("one.txt"), "one\n").unwrap();
    std::fs::write(fixture.alpha.join("pic.png"), "not really").unwrap();
    git(&fixture.alpha, &["add", "."]);
    git(&fixture.alpha, &["commit", "-q", "-m", "two files"]);
    let history = naran_core::history(&fixture.alpha, 1, naran_core::Refs::VISIBLE).unwrap();
    let hash = history.commits[0].hash.clone();
    let files = [file("one.txt", 'A'), file("pic.png", 'A')];
    let key = CommitKey {
        repository: fixture.alpha,
        hash: hash.clone(),
    };
    let missing = CommitKey {
        repository: "/missing".into(),
        hash,
    };

    let rows = read_commit_stack(&key, &files, 3).unwrap();

    assert!(rows.contains(&StackRow::Line("+one".into())));
    assert!(rows.contains(&StackRow::Note(IMAGE_NOTE.into())));
    assert_eq!(
        rows.last(),
        Some(&StackRow::Note("… and 3 more files are not shown.".into()))
    );
    assert!(read_commit_stack(&missing, &files[..1], 0).is_err());
}
