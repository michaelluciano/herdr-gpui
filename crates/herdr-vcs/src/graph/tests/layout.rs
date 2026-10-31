use super::*;

fn lanes(layout: &Layout) -> Vec<usize> {
    layout.rows.iter().map(|row| row.lane).collect()
}

fn edge(from_lane: usize, to_lane: usize, kind: EdgeKind) -> Edge {
    Edge {
        from_lane,
        to_lane,
        kind,
    }
}

#[test]
fn an_empty_window_has_no_lanes() {
    let layout = layout(&[]);
    assert!(layout.rows.is_empty());
    assert_eq!(layout.lane_count, 0);
}

#[test]
fn linear_history_stays_in_one_lane() {
    let layout = layout(&[commit("a", &["b"]), commit("b", &["c"]), commit("c", &[])]);
    assert_eq!(lanes(&layout), [0, 0, 0]);
    assert_eq!(layout.lane_count, 1);
    assert_eq!(layout.rows[0].edges, [edge(0, 0, EdgeKind::Parent)]);
    assert!(!layout.rows[0].has_incoming);
    assert!(layout.rows[1].has_incoming);
    assert!(layout.rows[2].edges.is_empty(), "a root has no edges");
    assert!(layout.rows.iter().all(|row| row.passing.is_empty()));
}

#[test]
fn a_branch_and_its_merge_use_two_lanes() {
    let layout = layout(&[
        commit("m", &["a", "b"]),
        commit("a", &["r"]),
        commit("b", &["r"]),
        commit("r", &[]),
    ]);
    assert_eq!(lanes(&layout), [0, 0, 1, 0]);
    assert_eq!(layout.lane_count, 2);
    assert_eq!(
        layout.rows[0].edges,
        [edge(0, 0, EdgeKind::Parent), edge(0, 1, EdgeKind::Merge)]
    );
    assert_eq!(layout.rows[1].passing, [1]);
    assert_eq!(layout.rows[2].passing, [0]);
    assert_eq!(layout.rows[2].edges, [edge(1, 0, EdgeKind::Parent)]);
    assert!(layout.rows[3].passing.is_empty());
}

#[test]
fn an_octopus_merge_fans_out_to_each_parent() {
    let layout = layout(&[
        commit("m", &["a", "b", "c"]),
        commit("a", &["r"]),
        commit("b", &["r"]),
        commit("c", &["r"]),
        commit("r", &[]),
    ]);
    assert_eq!(layout.rows[0].edges.len(), 3);
    assert_eq!(
        layout.rows[0].edges,
        [
            edge(0, 0, EdgeKind::Parent),
            edge(0, 1, EdgeKind::Merge),
            edge(0, 2, EdgeKind::Merge)
        ]
    );
    assert_eq!(lanes(&layout), [0, 0, 1, 2, 0]);
    assert_eq!(layout.lane_count, 3);
}

#[test]
fn a_lane_freed_by_a_merge_is_reused() {
    let layout = layout(&[
        commit("m", &["a", "b"]),
        commit("a", &["r"]),
        commit("b", &["r"]),
        commit("t", &["r"]),
        commit("r", &[]),
    ]);
    // After b folds into lane 0, lane 1 is free for the next new tip.
    assert_eq!(layout.rows[3].lane, 1);
    assert_eq!(layout.lane_count, 2);
}

#[test]
fn a_root_commit_has_a_lane_and_no_edges() {
    let layout = layout(&[commit("a", &[])]);
    assert_eq!(lanes(&layout), [0]);
    assert!(layout.rows[0].edges.is_empty());
    assert_eq!(layout.lane_count, 1);
}

#[test]
fn two_roots_share_or_split_lanes_by_overlap() {
    let sequential = layout(&[commit("a", &[]), commit("b", &[])]);
    assert_eq!(lanes(&sequential), [0, 0]);
    assert_eq!(sequential.lane_count, 1);
    let overlapping = layout(&[
        commit("x", &["a"]),
        commit("y", &["b"]),
        commit("a", &[]),
        commit("b", &[]),
    ]);
    assert_eq!(lanes(&overlapping), [0, 1, 0, 1]);
    assert_eq!(overlapping.lane_count, 2);
    assert_eq!(overlapping.rows[2].passing, [1]);
}

#[test]
fn a_long_lived_side_branch_keeps_its_lane() {
    let layout = layout(&[
        commit("merge", &["m4", "s3"]),
        commit("m4", &["m3"]),
        commit("s3", &["s2"]),
        commit("m3", &["m2"]),
        commit("s2", &["s1"]),
        commit("m2", &["base"]),
        commit("s1", &["base"]),
        commit("base", &[]),
    ]);
    assert_eq!(lanes(&layout), [0, 0, 1, 0, 1, 0, 1, 0]);
    assert_eq!(layout.lane_count, 2);
    for row in &layout.rows[1..7] {
        assert_eq!(row.passing.len(), 1, "the other lane passes through");
    }
}

#[test]
fn parents_outside_the_window_dangle() {
    let layout = layout(&[commit("a", &["gone"]), commit("b", &["c", "gone2"])]);
    assert_eq!(layout.rows[0].edges, [edge(0, 0, EdgeKind::Dangling)]);
    assert_eq!(layout.rows[1].lane, 0);
    assert!(
        layout.rows[1]
            .edges
            .iter()
            .any(|e| e.kind == EdgeKind::Dangling)
    );
}

#[test]
fn a_dangling_first_parent_frees_the_lane() {
    let layout = layout(&[commit("a", &["gone"]), commit("b", &[])]);
    assert_eq!(lanes(&layout), [0, 0]);
}

#[test]
fn duplicate_parents_draw_one_edge() {
    let layout = layout(&[commit("m", &["a", "a"]), commit("a", &[])]);
    assert_eq!(layout.rows[0].edges, [edge(0, 0, EdgeKind::Parent)]);
    assert_eq!(layout.lane_count, 1);
}

#[test]
fn a_parent_listed_before_its_child_does_not_panic() {
    let layout = layout(&[commit("a", &[]), commit("b", &["a"])]);
    assert_eq!(layout.rows.len(), 2);
    assert_eq!(layout.rows[1].edges[0].kind, EdgeKind::Dangling);
}

#[test]
fn layout_is_deterministic_and_lanes_stay_in_range() {
    let commits = [
        commit("m", &["a", "b", "c"]),
        commit("a", &["r"]),
        commit("b", &["r", "c"]),
        commit("c", &["r"]),
        commit("r", &[]),
    ];
    let first = layout(&commits);
    assert_eq!(first, layout(&commits));
    for row in &first.rows {
        assert!(row.lane < first.lane_count);
        assert!(row.passing.iter().all(|lane| *lane < first.lane_count));
        assert!(row.edges.iter().all(|e| e.to_lane < first.lane_count));
    }
}
