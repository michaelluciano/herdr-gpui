//! Drawing geometry of one graph row, in pixels local to the row: x runs
//! from 0 to the gutter width and y from 0 (row top) to the row height. Pure
//! and deterministic, so painting and hit testing share one source.

use herdr_vcs::graph::{EdgeKind, Row};

/// Where the short stub of a dangling edge ends, as a fraction of the row
/// height measured from the top: below the dot, well short of the bottom.
const DANGLING_END: f32 = 0.75;
/// Dot radius as a fraction of the row height, before clamping.
const DOT_RATIO: f32 = 0.2;
const DOT_MAX: f32 = 5.0;
const DOT_MIN: f32 = 2.0;

/// One stroke of a row. `lane` is the lane the stroke belongs to, so the
/// painter can pick the lane's color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Segment {
    /// A line to a parent outside the loaded window; it stops inside the row.
    Dangling {
        lane: usize,
        x0: f32,
        x1: f32,
        y0: f32,
        y1: f32,
    },
    /// The commit dot.
    Dot {
        cx: f32,
        cy: f32,
        lane: usize,
        r: f32,
    },
    /// A connector, vertical or diagonal.
    Line {
        lane: usize,
        x0: f32,
        x1: f32,
        y0: f32,
        y1: f32,
    },
}

/// Width of the graph gutter for `lane_count` lanes.
pub(crate) fn gutter_width(lane_count: usize, lane_width: f32) -> f32 {
    lane_count as f32 * sane(lane_width)
}

/// Lane width and gutter width for `lane_count` lanes: lanes keep their
/// natural width until the gutter reaches its cap, then shrink to fit down to
/// a floor, past which the gutter grows again.
pub(crate) fn lane_metrics(lane_count: usize) -> (f32, f32) {
    const NATURAL: f32 = 14.0;
    const GUTTER_CAP: f32 = 168.0;
    const LANE_FLOOR: f32 = 4.0;
    if lane_count == 0 {
        return (NATURAL, 0.0);
    }
    let lane = if gutter_width(lane_count, NATURAL) <= GUTTER_CAP {
        NATURAL
    } else {
        (GUTTER_CAP / lane_count as f32).max(LANE_FLOOR)
    };
    (lane, gutter_width(lane_count, lane))
}

/// Strokes for `row`: passing verticals, the incoming line, one connector per
/// edge, then the dot on top. Non-finite or negative sizes count as zero.
pub(crate) fn row_geometry(row: &Row, lane_width: f32, row_height: f32) -> Vec<Segment> {
    let lane_width = sane(lane_width);
    let height = sane(row_height);
    let middle = height / 2.0;
    let x = |lane: usize| lane as f32 * lane_width + lane_width / 2.0;
    let dot_x = x(row.lane);

    let mut segments = Vec::with_capacity(row.passing.len() + row.edges.len() + 2);
    for &lane in &row.passing {
        segments.push(Segment::Line {
            lane,
            x0: x(lane),
            x1: x(lane),
            y0: 0.0,
            y1: height,
        });
    }
    if row.has_incoming {
        segments.push(Segment::Line {
            lane: row.lane,
            x0: dot_x,
            x1: dot_x,
            y0: 0.0,
            y1: middle,
        });
    }
    for edge in &row.edges {
        let (x0, x1) = (x(edge.from_lane), x(edge.to_lane));
        segments.push(match edge.kind {
            EdgeKind::Dangling => Segment::Dangling {
                lane: edge.to_lane,
                x0,
                x1,
                y0: middle,
                y1: height * DANGLING_END,
            },
            EdgeKind::Merge | EdgeKind::Parent => Segment::Line {
                lane: edge.to_lane,
                x0,
                x1,
                y0: middle,
                y1: height,
            },
        });
    }
    let radius = (height * DOT_RATIO)
        .clamp(DOT_MIN, DOT_MAX)
        .min(height / 2.0)
        .min(lane_width / 2.0);
    segments.push(Segment::Dot {
        cx: dot_x,
        cy: middle,
        lane: row.lane,
        r: radius,
    });
    segments
}

fn sane(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
