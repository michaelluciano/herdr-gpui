use super::*;
use crate::graph_view::geometry::{Segment, gutter_width, row_geometry};
use herdr_vcs::graph::{Edge, EdgeKind, Row, layout};

const LW: f32 = 16.0;
const RH: f32 = 24.0;

fn x(lane: usize) -> f32 {
    lane as f32 * LW + LW / 2.0
}

fn edge(from_lane: usize, kind: EdgeKind, to_lane: usize) -> Edge {
    Edge {
        from_lane,
        kind,
        to_lane,
    }
}

fn lines(segments: &[Segment]) -> Vec<(f32, f32, f32, f32)> {
    segments
        .iter()
        .filter_map(|segment| match segment {
            Segment::Line { x0, y0, x1, y1, .. } => Some((*x0, *y0, *x1, *y1)),
            _ => None,
        })
        .collect()
}

fn dots(segments: &[Segment]) -> Vec<(f32, f32, f32)> {
    segments
        .iter()
        .filter_map(|segment| match segment {
            Segment::Dot { cx, cy, r, .. } => Some((*cx, *cy, *r)),
            _ => None,
        })
        .collect()
}

fn dangling(segments: &[Segment]) -> Vec<(f32, f32, f32, f32)> {
    segments
        .iter()
        .filter_map(|segment| match segment {
            Segment::Dangling { x0, y0, x1, y1, .. } => Some((*x0, *y0, *x1, *y1)),
            _ => None,
        })
        .collect()
}

fn assert_in_bounds(segments: &[Segment], gutter: f32) {
    let inside = |value: f32, max: f32| value.is_finite() && (0.0..=max).contains(&value);
    for segment in segments {
        let points = match *segment {
            Segment::Line { x0, y0, x1, y1, .. } | Segment::Dangling { x0, y0, x1, y1, .. } => {
                vec![(x0, y0), (x1, y1)]
            }
            Segment::Dot { cx, cy, r, .. } => {
                assert!(r.is_finite() && r > 0.0, "radius {r}");
                vec![(cx, cy)]
            }
        };
        for (px, py) in points {
            assert!(inside(px, gutter), "x {px} outside 0..={gutter}");
            assert!(inside(py, RH), "y {py} outside 0..={RH}");
        }
    }
}

#[test]
fn gutter_is_lane_count_times_lane_width() {
    assert_eq!(gutter_width(0, LW), 0.0);
    assert_eq!(gutter_width(1, LW), LW);
    assert_eq!(gutter_width(5, LW), 5.0 * LW);
}

#[test]
fn zero_edges_draws_only_the_dot() {
    let row = Row::default();
    let segments = row_geometry(&row, LW, RH);
    assert_eq!(segments.len(), 1);
    assert_eq!(
        dots(&segments),
        vec![(x(0), RH / 2.0, dots(&segments)[0].2)]
    );
    assert_in_bounds(&segments, gutter_width(1, LW));
}

#[test]
fn dot_radius_is_clamped_to_a_sane_range() {
    let radius = |row_height: f32| dots(&row_geometry(&Row::default(), LW, row_height))[0].2;
    assert!(radius(4.0) > 0.0);
    assert!(radius(4.0) <= 2.0);
    assert!(radius(1000.0) <= 8.0);
    assert!(radius(1000.0) >= radius(24.0));
}

#[test]
fn linear_history_connects_top_to_dot_to_bottom() {
    let graph = layout(&[commit("a", &["b"]), commit("b", &["c"]), commit("c", &[])]);
    let middle = row_geometry(&graph.rows[1], LW, RH);
    assert_eq!(
        lines(&middle),
        vec![(x(0), 0.0, x(0), RH / 2.0), (x(0), RH / 2.0, x(0), RH)]
    );
    assert_eq!(dots(&middle).len(), 1);
    // The first row has no incoming line; the last has no outgoing edge.
    assert_eq!(lines(&row_geometry(&graph.rows[0], LW, RH)).len(), 1);
    assert_eq!(lines(&row_geometry(&graph.rows[2], LW, RH)).len(), 1);
    assert_in_bounds(&middle, gutter_width(graph.lane_count, LW));
}

#[test]
fn fork_draws_diagonals_between_lanes() {
    // Two tips share a parent: the second tip opens lane 1 and joins at c.
    let graph = layout(&[commit("a", &["c"]), commit("b", &["c"]), commit("c", &[])]);
    let gutter = gutter_width(graph.lane_count, LW);
    let tip = row_geometry(&graph.rows[1], LW, RH);
    let to_bottom: Vec<_> = lines(&tip)
        .into_iter()
        .filter(|(_, y0, _, y1)| *y0 == RH / 2.0 && *y1 == RH)
        .collect();
    assert_eq!(to_bottom.len(), 1);
    assert_in_bounds(&tip, gutter);

    let diagonal = Row {
        edges: vec![edge(1, EdgeKind::Parent, 0)],
        has_incoming: false,
        lane: 1,
        passing: vec![],
    };
    let segments = row_geometry(&diagonal, LW, RH);
    assert!(lines(&segments).contains(&(x(1), RH / 2.0, x(0), RH)));
    assert_in_bounds(&segments, gutter_width(2, LW));
}

#[test]
fn merge_and_parent_edges_both_draw() {
    let row = Row {
        edges: vec![edge(0, EdgeKind::Parent, 0), edge(0, EdgeKind::Merge, 1)],
        has_incoming: true,
        lane: 0,
        passing: vec![],
    };
    let segments = row_geometry(&row, LW, RH);
    let found = lines(&segments);
    assert!(found.contains(&(x(0), 0.0, x(0), RH / 2.0)));
    assert!(found.contains(&(x(0), RH / 2.0, x(0), RH)));
    assert!(found.contains(&(x(0), RH / 2.0, x(1), RH)));
    assert_in_bounds(&segments, gutter_width(2, LW));
}

#[test]
fn octopus_merge_draws_one_line_per_parent() {
    let graph = layout(&[
        commit("m", &["a", "b", "c", "d"]),
        commit("a", &[]),
        commit("b", &[]),
        commit("c", &[]),
        commit("d", &[]),
    ]);
    let segments = row_geometry(&graph.rows[0], LW, RH);
    let outgoing = lines(&segments)
        .into_iter()
        .filter(|(_, y0, _, y1)| *y0 == RH / 2.0 && *y1 == RH)
        .count();
    assert_eq!(outgoing, 4);
    assert_in_bounds(&segments, gutter_width(graph.lane_count, LW));
}

#[test]
fn dangling_edge_stops_inside_the_row_and_is_not_a_line() {
    let graph = layout(&[commit("a", &["missing"])]);
    let segments = row_geometry(&graph.rows[0], LW, RH);
    let stubs = dangling(&segments);
    assert_eq!(stubs.len(), 1);
    let (x0, y0, x1, y1) = stubs[0];
    assert_eq!((x0, y0, x1), (x(0), RH / 2.0, x(0)));
    assert!(y1 > y0 && y1 < RH);
    assert!(lines(&segments).is_empty());
    assert_in_bounds(&segments, gutter_width(graph.lane_count.max(1), LW));
}

#[test]
fn passing_lanes_draw_full_height_verticals() {
    let row = Row {
        edges: vec![edge(1, EdgeKind::Parent, 1)],
        has_incoming: false,
        lane: 1,
        passing: vec![0, 2],
    };
    let segments = row_geometry(&row, LW, RH);
    let found = lines(&segments);
    assert!(found.contains(&(x(0), 0.0, x(0), RH)));
    assert!(found.contains(&(x(2), 0.0, x(2), RH)));
    assert_in_bounds(&segments, gutter_width(3, LW));
}

#[test]
fn high_lane_is_positioned_by_the_same_formula() {
    let row = Row {
        edges: vec![edge(9, EdgeKind::Parent, 9)],
        has_incoming: true,
        lane: 9,
        passing: vec![0],
    };
    let segments = row_geometry(&row, LW, RH);
    assert_eq!(dots(&segments)[0].0, x(9));
    assert!(lines(&segments).contains(&(x(9), 0.0, x(9), RH / 2.0)));
    assert_in_bounds(&segments, gutter_width(10, LW));
}

#[test]
fn geometry_is_deterministic() {
    let graph = layout(&[
        commit("m", &["a", "b"]),
        commit("a", &["c"]),
        commit("b", &["c"]),
        commit("c", &[]),
    ]);
    for row in &graph.rows {
        assert_eq!(row_geometry(row, LW, RH), row_geometry(row, LW, RH));
    }
}

#[test]
fn segments_carry_the_lane_they_belong_to() {
    let row = Row {
        edges: vec![edge(1, EdgeKind::Merge, 2)],
        has_incoming: true,
        lane: 1,
        passing: vec![0],
    };
    let segments = row_geometry(&row, LW, RH);
    assert!(
        segments
            .iter()
            .any(|s| matches!(s, Segment::Dot { lane: 1, .. }))
    );
    assert!(
        segments
            .iter()
            .any(|s| matches!(s, Segment::Line { lane: 0, .. }))
    );
    assert!(
        segments
            .iter()
            .any(|s| matches!(s, Segment::Line { lane: 2, .. }))
    );
}
