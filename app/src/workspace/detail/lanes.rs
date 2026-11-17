//! The lanes of the commit graph: how wide the column is, which commit HEAD
//! is on, and where the dashed link from the Uncommitted changes row runs.

use gpui::{Hsla, px};
use naran_core::graph;
use naran_core::log::Commit;

/// Width of one lane of the graph.
pub(super) const LANE_WIDTH: f32 = 14.;
/// The graph column is at least this wide: its header label must fit.
const GRAPH_MINIMUM: f32 = 48.;
/// The graph shows at most this many lanes.
pub(super) const MAX_LANES: usize = 12;

/// Width of the graph column: as many lanes as the loaded commits use, and
/// the lane the Uncommitted changes link runs in.
pub(super) fn graph_width(rows: &[graph::Row], link: Option<HeadLink>) -> f32 {
    let lanes = rows
        .iter()
        .flat_map(lanes_of)
        .chain(link.map(|link| link.link_lane))
        .max()
        .map_or(1, |lane| lane + 1)
        .min(MAX_LANES);
    // Never narrower than the "Graph" label above it.
    (f32::from(px(LANE_WIDTH) * lanes) + 4.).max(GRAPH_MINIMUM)
}

/// Every lane a row touches: lines coming in, passing through, going out, and its dot.
fn lanes_of(row: &graph::Row) -> impl Iterator<Item = usize> + '_ {
    let lines = row.incoming.iter().chain(&row.through).chain(&row.outgoing);
    lines.copied().chain([row.lane])
}

/// Is this label the one of HEAD: `HEAD` (detached) or `HEAD -> branch`?
pub(super) fn is_head_ref(name: &str) -> bool {
    name == "HEAD" || name.starts_with("HEAD -> ")
}

/// Index of the commit HEAD is on, when it is in the list.
pub(super) fn head_position(commits: &[Commit]) -> Option<usize> {
    commits
        .iter()
        .position(|commit| commit.refs.iter().any(|name| is_head_ref(name)))
}

/// Where the Uncommitted changes row joins the graph: the commit that HEAD is
/// on, the lane its dot sits in, and the lane the dashed link runs down in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HeadLink {
    /// Index of the HEAD commit in the commit list.
    pub(super) position: usize,
    pub(super) head_lane: usize,
    pub(super) link_lane: usize,
}

/// Finds the HEAD commit (the one labeled `HEAD` or `HEAD -> branch`). The link
/// runs straight down the HEAD lane when nothing else uses that lane above
/// the commit; otherwise it runs down a lane of its own at the right, so it
/// never lies over another line. `None` when HEAD is not in the list.
pub(super) fn head_link(commits: &[Commit], rows: &[graph::Row]) -> Option<HeadLink> {
    let position = head_position(commits)?;
    rows.get(position).map(|row| HeadLink {
        position,
        head_lane: row.lane,
        link_lane: link_lane(&rows[..position], row),
    })
}

/// The lane of the link down to `head`, given the rows `above` it.
fn link_lane(above: &[graph::Row], head: &graph::Row) -> usize {
    let busy = above
        .iter()
        .any(|row| lanes_of(row).any(|lane| lane == head.lane));
    if !busy {
        return head.lane;
    }
    let widest = above.iter().chain([head]).flat_map(lanes_of).max();
    (widest.unwrap_or(0) + 1).min(MAX_LANES - 1)
}

/// The part of the dashed link inside one commit row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LinkPiece {
    /// The link passes by this row. `head_lane` gives it its color.
    Through { lane: usize, head_lane: usize },
    /// The link ends in this row, at the HEAD commit's dot.
    End { from_lane: usize, to_lane: usize },
}

/// Colors of the lanes, in order, then again.
pub(super) fn lane_color(lane: usize) -> Hsla {
    const COLORS: [[u8; 3]; 8] = [
        [0x7a, 0xa2, 0xf7],
        [0x9e, 0xce, 0x6a],
        [0xe0, 0xaf, 0x68],
        [0xf7, 0x76, 0x8e],
        [0xbb, 0x9a, 0xf7],
        [0x7d, 0xcf, 0xff],
        [0xff, 0x9e, 0x64],
        [0x73, 0xda, 0xca],
    ];
    crate::theme::color(COLORS[lane % COLORS.len()])
}

/// The graph part of one commit row: lanes passing through, lanes joining the
/// commit from above and leaving it downward, and the commit's dot.
pub(super) fn link_piece(link: HeadLink, position: usize) -> Option<LinkPiece> {
    match position.cmp(&link.position) {
        std::cmp::Ordering::Less => Some(LinkPiece::Through {
            lane: link.link_lane,
            head_lane: link.head_lane,
        }),
        std::cmp::Ordering::Equal => Some(LinkPiece::End {
            from_lane: link.link_lane,
            to_lane: link.head_lane,
        }),
        std::cmp::Ordering::Greater => None,
    }
}
