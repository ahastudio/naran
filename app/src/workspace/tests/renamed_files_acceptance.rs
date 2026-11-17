//! How a renamed file shows in the Staged list, driven through the window.

use super::staging_acceptance::listed;
use super::*;

#[gpui::test]
fn a_renamed_file_shows_its_new_path_then_the_old_one_and_the_others_show_their_path(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    git_output(&fixture.alpha, &["mv", "README.md", "GUIDE.md"]);
    std::fs::write(fixture.alpha.join("loose.txt"), "one\n").unwrap();
    let (_workspace, cx) = open(&fixture, cx);

    assert!(listed(cx, "label-staged-GUIDE.md <- README.md"));
    assert!(listed(cx, "label-unstaged-loose.txt"));
    assert!(!listed(cx, "label-staged-GUIDE.md"));
}
