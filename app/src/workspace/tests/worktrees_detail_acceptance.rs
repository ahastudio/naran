//! The detail of a worktree, driven through the window: the bar that names
//! its repository, and the selection when the worktree is removed.

use super::staging_acceptance::{click_row, listed};
use super::worktrees_acceptance::{add_worktree, open_selected_worktree, row, toggle};
use super::*;
use gpui::Modifiers;

#[gpui::test]
fn the_detail_of_a_worktree_says_which_repository_it_belongs_to(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let (workspace, cx, _folder) = open_selected_worktree(&fixture, cx);
    let has_banner = |cx: &mut VisualTestContext| {
        workspace.read_with(cx, |workspace, _| {
            workspace
                .worktree_banner(&crate::theme::Theme::dark())
                .is_some()
        })
    };

    assert!(listed(cx, "worktree-banner"));
    assert!(has_banner(cx));

    // Bounds of what was drawn stay in the window, so the second answer is
    // read from the workspace.
    click_row(cx, toggle(&fixture.alpha), Modifiers::none());

    assert!(!has_banner(cx));
}

#[gpui::test]
fn the_detail_of_a_missing_worktree_names_it_without_a_branch(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    let folder = add_worktree(&fixture.alpha, "feature");
    std::fs::remove_dir_all(&folder).unwrap();
    let (workspace, cx) = open(&fixture, cx);
    click_row(cx, toggle(&fixture.alpha), Modifiers::none());

    click_row(cx, row(&folder), Modifiers::none());

    workspace.read_with(cx, |workspace, _| {
        assert!(
            workspace
                .worktree_banner(&crate::theme::Theme::dark())
                .is_some()
        );
    });
}

#[gpui::test]
fn a_worktree_removed_while_selected_moves_the_selection_on_a_partial_refresh(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(&[]);
    let (workspace, cx, folder) = open_selected_worktree(&fixture, cx);
    git(
        &fixture.alpha,
        &["worktree", "remove", "--force", &folder.to_string_lossy()],
    );

    let alpha = vec![fixture.alpha.clone()];
    workspace.update(cx, |_, cx| Workspace::refresh_some(alpha, cx));
    cx.run_until_parked();

    assert_eq!(selected(&workspace, cx), Some(fixture.alpha.clone()));
}
