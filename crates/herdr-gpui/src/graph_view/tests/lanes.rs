use crate::graph_view::geometry::lane_metrics;

#[test]
fn small_lane_counts_keep_the_natural_width() {
    assert_eq!(lane_metrics(3), (14.0, 42.0));
    assert_eq!(lane_metrics(1), (14.0, 14.0));
}

#[test]
fn six_lanes_still_fit_at_the_natural_width() {
    assert_eq!(lane_metrics(6), (14.0, 84.0));
}

#[test]
fn above_the_cap_lanes_shrink_to_fit() {
    let (lane, gutter) = lane_metrics(7);
    assert!((lane - 96.0 / 7.0).abs() < 0.001, "lane width {lane}");
    assert!((gutter - 96.0).abs() < 0.01, "gutter {gutter}");
}

#[test]
fn the_floor_is_reached_at_twenty_four_lanes() {
    let (lane, gutter) = lane_metrics(24);
    assert!((lane - 4.0).abs() < 0.001, "lane width {lane}");
    assert!((gutter - 96.0).abs() < 0.01, "gutter {gutter}");
}

#[test]
fn lanes_never_shrink_below_the_floor() {
    let (lane, gutter) = lane_metrics(30);
    assert!((lane - 4.0).abs() < 0.001 && (gutter - 120.0).abs() < 0.01);
    let (lane, gutter) = lane_metrics(100);
    assert_eq!(lane, 4.0);
    assert_eq!(gutter, 400.0);
}

#[test]
fn zero_lanes_do_not_panic_or_produce_nan() {
    let (lane, gutter) = lane_metrics(0);
    assert!(lane.is_finite() && gutter.is_finite());
    assert!(lane >= 0.0 && gutter >= 0.0);
    assert!(gutter <= 96.0);
}

#[test]
fn lane_width_never_grows_and_gutter_never_shrinks_with_more_lanes() {
    let mut previous = lane_metrics(1);
    for count in 2..=120 {
        let current = lane_metrics(count);
        assert!(current.0 <= previous.0, "lane width grew at {count}");
        assert!(current.1 >= previous.1 - 0.01, "gutter shrank at {count}");
        previous = current;
    }
}
