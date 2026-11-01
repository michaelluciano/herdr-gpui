use super::*;

#[test]
fn the_graph_kind_and_operation_are_named_and_timed_like_the_other_tabs() {
    assert_eq!(TabKind::Graph.name(), "graph");
    assert!(!TabKind::Graph.is_page());
    assert_eq!(TabOp::Graph.wait_seconds(), ARRIVE_WAIT_SECONDS);
}

#[test]
fn the_graph_operation_has_a_stable_wire_form() {
    assert_eq!(
        serde_json::to_string(&TabOp::Graph).unwrap(),
        r#"{"op":"graph"}"#
    );
    assert_eq!(
        serde_json::from_str::<TabOp>(r#"{"op":"graph"}"#).unwrap(),
        TabOp::Graph
    );
    round_trip(TabOp::Graph);
}
