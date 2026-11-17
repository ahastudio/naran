//! Drawing the graph cell of a row of the commit list on a canvas: the lines
//! of the lanes, the dashed link from the Uncommitted changes row, the dots.

use gpui::{AnyElement, Hsla, Pixels, Window, div, prelude::*, px};
use naran_core::graph;

use super::HISTORY_ROW;
use super::lanes::{HeadLink, LANE_WIDTH, LinkPiece, MAX_LANES, lane_color};

/// A dashed line, in the color of the HEAD lane.
fn dashed(window: &mut Window, from: gpui::Point<Pixels>, to: gpui::Point<Pixels>, lane: usize) {
    let path = gpui::PathBuilder::stroke(px(1.5)).dash_array(&[px(3.), px(3.)]);
    paint_line(window, path, (from, to), lane);
}

fn stroke(window: &mut Window, from: gpui::Point<Pixels>, to: gpui::Point<Pixels>, lane: usize) {
    paint_line(window, gpui::PathBuilder::stroke(px(1.5)), (from, to), lane);
}

fn paint_line(
    window: &mut Window,
    mut path: gpui::PathBuilder,
    (from, to): (gpui::Point<Pixels>, gpui::Point<Pixels>),
    lane: usize,
) {
    path.move_to(from);
    path.line_to(to);
    path.build()
        .map(|path| window.paint_path(path, lane_color(lane)))
        .ok();
}

/// The two ends of a line and the lane it is drawn in.
type Segment = (gpui::Point<Pixels>, gpui::Point<Pixels>, usize);

/// The box of a graph cell, with the places lines and dots are drawn at.
#[derive(Clone, Copy)]
struct CellBox(gpui::Bounds<Pixels>);

impl CellBox {
    fn point(&self, lane: usize, y: Pixels) -> gpui::Point<Pixels> {
        let x = self.0.origin.x + px(LANE_WIDTH) * lane.min(MAX_LANES - 1) + px(LANE_WIDTH / 2.);
        gpui::point(x, y)
    }

    /// A line from the top to the bottom of a lane.
    fn straight(&self, lane: usize) -> Segment {
        (
            self.point(lane, self.top()),
            self.point(lane, self.bottom()),
            lane,
        )
    }

    fn top(&self) -> Pixels {
        self.0.origin.y
    }

    fn middle(&self) -> Pixels {
        self.0.origin.y + self.0.size.height / 2.
    }

    fn bottom(&self) -> Pixels {
        self.0.origin.y + self.0.size.height
    }
}

/// A round dot with a ring.
struct Dot {
    center: gpui::Point<Pixels>,
    radius: Pixels,
    fill: Hsla,
    ring: (Pixels, Hsla),
}

fn paint_dot(window: &mut Window, dot: &Dot) {
    let (center, radius) = (dot.center, dot.radius);
    let bounds = gpui::Bounds::new(
        gpui::point(center.x - radius, center.y - radius),
        gpui::size(radius * 2., radius * 2.),
    );
    let quad = gpui::fill(bounds, dot.fill).corner_radii(radius);
    window.paint_quad(quad.border_widths(dot.ring.0).border_color(dot.ring.1));
}

fn empty_graph_cell(width: f32) -> AnyElement {
    div()
        .flex_none()
        .w(px(width))
        .h(px(HISTORY_ROW))
        .into_any_element()
}

/// A graph cell drawn on a canvas.
fn graph_canvas(width: f32, paint: impl Fn(CellBox, &mut Window) + 'static) -> AnyElement {
    gpui::canvas(
        |_, _, _| (),
        move |bounds, (), window, _| paint(CellBox(bounds), window),
    )
    .flex_none()
    .w(px(width))
    .h(px(HISTORY_ROW))
    .into_any_element()
}

/// The graph part of the Uncommitted changes row: a hollow dot in the link lane
/// and a dashed line down into the commits below.
pub(super) fn uncommitted_cell(link: Option<HeadLink>, width: f32, background: Hsla) -> AnyElement {
    let Some(link) = link else {
        return empty_graph_cell(width);
    };
    graph_canvas(width, move |cell, window| {
        paint_uncommitted(cell, window, link, background);
    })
}

fn paint_uncommitted(cell: CellBox, window: &mut Window, link: HeadLink, background: Hsla) {
    let from = cell.point(link.link_lane, cell.middle());
    dashed(
        window,
        from,
        cell.point(link.link_lane, cell.bottom()),
        link.head_lane,
    );
    // Hollow: a ring in the lane color around the background.
    let ring = (px(1.5), lane_color(link.head_lane));
    paint_dot(
        window,
        &Dot {
            center: from,
            radius: px(4.5),
            fill: background,
            ring,
        },
    );
}

/// What is drawn in the graph cell of one commit row.
pub(super) struct GraphCell {
    pub(super) row: graph::Row,
    pub(super) background: Hsla,
    pub(super) piece: Option<LinkPiece>,
    pub(super) head_ring: Option<Hsla>,
}

impl GraphCell {
    fn paint(&self, cell: CellBox, window: &mut Window) {
        for (from, to, lane) in self.segments(cell) {
            stroke(window, from, to, lane);
        }
        self.paint_link(cell, window);
        self.paint_commit(cell, window);
    }

    /// The lines of lanes passing through, joining the commit and leaving it.
    fn segments(&self, cell: CellBox) -> Vec<Segment> {
        let dot = cell.point(self.row.lane, cell.middle());
        let through = self.row.through.iter().map(|&lane| cell.straight(lane));
        let incoming = self
            .row
            .incoming
            .iter()
            .map(|&lane| (cell.point(lane, cell.top()), dot, lane));
        let outgoing = self
            .row
            .outgoing
            .iter()
            .map(|&lane| (dot, cell.point(lane, cell.bottom()), lane));
        through.chain(incoming).chain(outgoing).collect()
    }

    /// The dashed link from the Uncommitted changes row above.
    fn paint_link(&self, cell: CellBox, window: &mut Window) {
        match self.piece {
            Some(LinkPiece::Through { lane, head_lane }) => dashed(
                window,
                cell.point(lane, cell.top()),
                cell.point(lane, cell.bottom()),
                head_lane,
            ),
            Some(LinkPiece::End { from_lane, to_lane }) => dashed(
                window,
                cell.point(from_lane, cell.top()),
                cell.point(to_lane, cell.middle()),
                to_lane,
            ),
            None => {}
        }
    }

    /// The commit HEAD is on: a bigger dot with a bright ring, "you are here".
    fn paint_commit(&self, cell: CellBox, window: &mut Window) {
        let (radius, width) = self
            .head_ring
            .map_or((px(4.5), px(1.5)), |_| (px(6.5), px(2.5)));
        let ring = (width, self.head_ring.unwrap_or(self.background));
        let center = cell.point(self.row.lane, cell.middle());
        paint_dot(
            window,
            &Dot {
                center,
                radius,
                fill: lane_color(self.row.lane),
                ring,
            },
        );
    }
}

/// The graph cell of a commit row; an empty box when the commit has no row
/// of the graph.
pub(super) fn graph_cell(drawing: Option<GraphCell>, width: f32) -> AnyElement {
    let Some(drawing) = drawing else {
        return empty_graph_cell(width);
    };
    graph_canvas(width, move |cell, window| drawing.paint(cell, window))
}
