use super::*;
use crate::handoffs::{
    classify::Shelf,
    line::{detail, short_stamp},
};

#[test]
fn a_local_iso_created_shows_month_day_and_time() {
    assert_eq!(
        short_stamp("2026-10-31T16:37:41").as_deref(),
        Some("10-31 16:37")
    );
    assert_eq!(
        short_stamp("2026-01-02T03:04").as_deref(),
        Some("01-02 03:04"),
        "seconds are optional"
    );
}

#[test]
fn a_missing_or_malformed_created_has_no_stamp() {
    let malformed = [
        "",
        "   ",
        "2026-10-31 16:37:41",
        "2026-10-31T16:37:41Z",
        "2026-10-31T16:37:41.123",
        "2026-10-31T16",
        "2026-10-31T16:37:41:09",
        "2026-13-01T10:00:00",
        "2026-00-01T10:00:00",
        "2026-10-00T10:00:00",
        "2026-10-32T10:00:00",
        "2026-10-31T24:00:00",
        "2026-10-31T16:60:00",
        "2026-10-31T16:37:60",
        "26-10-31T16:37:41",
        "2026-1O-31T16:37:41",
        "garbage",
    ];
    for created in malformed {
        assert_eq!(short_stamp(created), None, "{created:?}");
    }
}

#[test]
fn a_row_line_is_the_project_then_the_stamp() {
    let handoff = Handoff {
        created: "2026-10-31T13:41:42".into(),
        ..fixture("S1", "claude-setup", "", "open")
    };
    assert_eq!(detail(&handoff, None), "claude-setup \u{b7} 10-31 13:41");
}

#[test]
fn a_date_only_created_shows_month_and_day() {
    assert_eq!(short_stamp("2026-10-31").as_deref(), Some("10-31"));
    assert_eq!(short_stamp("2026-01-02").as_deref(), Some("01-02"));
    for created in [
        "2026-13-01",
        "2026-00-01",
        "2026-10-00",
        "2026-10-32",
        "2026-1-01",
        "2026-10-3",
        "2026-10-31-01",
        "26-10-31",
    ] {
        assert_eq!(short_stamp(created), None, "{created:?}");
    }
}

#[test]
fn a_date_only_row_line_is_the_project_then_the_day() {
    let handoff = fixture("S1", "kb", "2026-10-31", "open");
    assert_eq!(detail(&handoff, None), "kb \u{b7} 10-31");
}

#[test]
fn a_row_without_a_usable_created_shows_the_project_only() {
    for created in ["", "not a date", "2026-13-01"] {
        let handoff = fixture("S1", "kb", created, "open");
        assert_eq!(detail(&handoff, None), "kb", "{created:?}");
    }
}

#[test]
fn a_history_row_ends_with_why_it_is_there() {
    let dated = fixture("S1", "remora", "2026-10-29T10:00:00", "open");
    assert_eq!(
        detail(&dated, Some(Shelf::PickedUp)),
        "remora \u{b7} 10-29 10:00 \u{b7} picked up"
    );
    assert_eq!(
        detail(&dated, Some(Shelf::Superseded)),
        "remora \u{b7} 10-29 10:00 \u{b7} superseded"
    );
    let dated_day = fixture("S1", "kb", "2026-10-31", "claimed");
    assert_eq!(
        detail(&dated_day, Some(Shelf::PickedUp)),
        "kb \u{b7} 10-31 \u{b7} picked up"
    );
    let undated = fixture("S1", "kb", "not a date", "claimed");
    assert_eq!(
        detail(&undated, Some(Shelf::PickedUp)),
        "kb \u{b7} picked up"
    );
}
