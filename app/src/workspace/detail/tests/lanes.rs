//! The lanes of the commit graph and the dashed link from the Uncommitted
//! changes row down to the commit HEAD is on.

use gpui::{AnyElement, Render, Window, div, prelude::*, px};
use naran_core::graph;

use super::*;
use crate::theme::Theme;
use crate::workspace::detail::follow;
use crate::workspace::detail::lanes::{
    HeadLink, LinkPiece, graph_width, head_link, head_position, is_head_ref, link_piece,
};
use crate::workspace::detail::painting::{graph_cell, uncommitted_cell};
use crate::workspace::tests::git;

fn commit(hash: &str, parents: &[&str], refs: &[&str]) -> Commit {
    Commit {
        hash: hash.into(),
        parents: parents.iter().map(ToString::to_string).collect(),
        author: "n".into(),
        time: 0,
        refs: refs.iter().map(ToString::to_string).collect(),
        subject: hash.into(),
    }
}

fn link(commits: &[Commit]) -> Option<HeadLink> {
    head_link(commits, &graph::layout(commits))
}

#[test]
fn the_link_goes_straight_to_a_head_at_the_top() {
    let commits = [
        commit("c3", &["c2"], &["HEAD -> main"]),
        commit("c2", &["c1"], &[]),
        commit("c1", &[], &[]),
    ];
    assert_eq!(
        link(&commits),
        Some(HeadLink {
            position: 0,
            head_lane: 0,
            link_lane: 0
        })
    );
    assert_eq!(
        link_piece(link(&commits).unwrap(), 0),
        Some(LinkPiece::End {
            from_lane: 0,
            to_lane: 0
        })
    );
    assert_eq!(
        link_piece(link(&commits).unwrap(), 1),
        None,
        "nothing below the HEAD commit"
    );
}

#[test]
fn a_detached_head_counts_too() {
    let commits = [commit("b", &["a"], &["HEAD"]), commit("a", &[], &[])];
    assert_eq!(link(&commits).map(|link| link.position), Some(0));
}

#[test]
fn the_link_passes_the_commits_above_a_head_on_another_branch() {
    // main is ahead; HEAD is on the feature branch, in its own lane.
    let commits = [
        commit("m1", &["m0"], &["main"]),
        commit("f2", &["f1"], &["HEAD -> feature"]),
        commit("f1", &["m0"], &[]),
        commit("m0", &[], &[]),
    ];
    let link = link(&commits).unwrap();
    assert_eq!(
        (link.position, link.head_lane, link.link_lane),
        (1, 1, 1),
        "lane 1 is free above the commit"
    );
    assert_eq!(
        link_piece(link, 0),
        Some(LinkPiece::Through {
            lane: 1,
            head_lane: 1
        })
    );
    assert_eq!(
        link_piece(link, 1),
        Some(LinkPiece::End {
            from_lane: 1,
            to_lane: 1
        })
    );
}

#[test]
fn the_link_takes_a_lane_of_its_own_when_the_head_lane_is_busy_above() {
    // The newest commit sits in the same lane as the HEAD commit below it.
    let commits = [
        commit("t", &["h"], &["other"]),
        commit("h", &[], &["HEAD -> main"]),
    ];
    let link = link(&commits).unwrap();
    assert_eq!((link.position, link.head_lane), (1, 0));
    assert_eq!(link.link_lane, 1, "not on top of the lines of lane 0");
    assert_eq!(
        link_piece(link, 1),
        Some(LinkPiece::End {
            from_lane: 1,
            to_lane: 0
        })
    );
}

#[test]
fn head_is_the_commit_labeled_head_or_head_arrow_branch() {
    let commits = [
        commit("c", &["b"], &["main"]),
        commit("b", &["a"], &["HEAD -> feature", "tag: v1"]),
        commit("a", &[], &[]),
    ];
    assert_eq!(head_position(&commits), Some(1));
    assert_eq!(
        head_position(&[commit("x", &[], &["HEAD"])]),
        Some(0),
        "a detached HEAD"
    );
    assert_eq!(
        head_position(&[commit("x", &[], &["HEADLINE", "origin/HEAD"])]),
        None,
        "other labels do not count"
    );
    assert!(is_head_ref("HEAD -> main") && is_head_ref("HEAD") && !is_head_ref("origin/HEAD"));
}

#[test]
fn a_row_scrolled_into_view_goes_to_the_edge_it_came_from() {
    assert_eq!(format!("{:?}", follow(1)), "Bottom");
    assert_eq!(format!("{:?}", follow(-1)), "Top");
}

#[test]
fn without_a_head_commit_in_the_list_there_is_no_link() {
    let commits = [commit("b", &["a"], &["main"]), commit("a", &[], &[])];
    assert_eq!(link(&commits), None);
}

#[test]
fn the_graph_is_wide_enough_for_the_link_lane() {
    let rows = graph::layout(&[commit("t", &["h"], &[]), commit("h", &[], &[])]);
    let narrow = graph_width(&rows, None);
    let wide = graph_width(
        &rows,
        Some(HeadLink {
            position: 1,
            head_lane: 0,
            link_lane: 5,
        }),
    );
    assert!(wide > narrow, "{wide} {narrow}");
}

#[gpui::test]
fn the_uncommitted_link_passes_the_commits_above_a_detached_head(cx: &mut TestAppContext) {
    let fixture = fixture(&[]);
    for name in ["two", "three"] {
        std::fs::write(fixture.alpha.join(format!("{name}.txt")), name).unwrap();
        git(&fixture.alpha, &["add", "."]);
        git(&fixture.alpha, &["commit", "-q", "-m", name]);
    }
    git(&fixture.alpha, &["checkout", "-q", "--detach", "HEAD~1"]);
    std::fs::write(fixture.alpha.join("loose.txt"), "loose").unwrap();
    let (_workspace, cx) = open(&fixture, cx);
    cx.run_until_parked();

    assert!(cx.debug_bounds("uncommitted-row").is_some());
    assert!(
        cx.debug_bounds("commit-row-0").is_some(),
        "the newer commit above HEAD"
    );
    assert!(
        cx.debug_bounds("commit-row-1").is_some(),
        "the commit HEAD is on"
    );
}

#[gpui::test]
fn the_graph_cells_without_a_row_or_a_link_are_empty_boxes(cx: &mut TestAppContext) {
    struct Cells;
    impl Render for Cells {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let background = Theme::dark().background;
            let boxed = |name: &'static str, cell: AnyElement| {
                div()
                    .flex_none()
                    .debug_selector(move || name.into())
                    .child(cell)
            };
            div()
                .flex()
                .child(boxed("no-row", graph_cell(None, 40.)))
                .child(boxed("no-link", uncommitted_cell(None, 52., background)))
        }
    }
    let (_, cx) = cx.add_window_view(|_, _| Cells);
    cx.run_until_parked();

    assert_eq!(cx.debug_bounds("no-row").unwrap().size.width, px(40.));
    assert_eq!(cx.debug_bounds("no-link").unwrap().size.width, px(52.));
}
