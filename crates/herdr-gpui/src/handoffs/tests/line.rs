use super::*;
use crate::handoffs::{
    classify::Shelf,
    line::{Created, age, detail, parse_created, short_id},
};
use chrono::{NaiveDate, NaiveDateTime};

#[test]
fn a_local_iso_created_parses_with_or_without_seconds() {
    let full = NaiveDate::from_ymd_opt(2026, 10, 31)
        .unwrap()
        .and_hms_opt(16, 37, 41)
        .unwrap();
    assert_eq!(
        parse_created("2026-10-31T16:37:41"),
        Some(Created::Moment(full))
    );
    assert_eq!(parse_created("2026-01-02T03:04"), Some(at(1, 2, 3, 4)));
}

#[test]
fn a_missing_or_malformed_created_does_not_parse() {
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
        assert_eq!(parse_created(created), None, "{created:?}");
    }
}

#[test]
fn a_date_only_created_parses_to_a_day() {
    assert_eq!(
        parse_created("2026-10-31"),
        Some(Created::Day(NaiveDate::from_ymd_opt(2026, 10, 31).unwrap()))
    );
    for created in [
        "2026-13-01",
        "2026-00-01",
        "2026-10-00",
        "2026-10-32",
        "2026-02-30",
        "2026-1-01",
        "2026-10-3",
        "2026-10-31-01",
        "26-10-31",
    ] {
        assert_eq!(parse_created(created), None, "{created:?}");
    }
}

fn at(month: u32, day: u32, hour: u32, minute: u32) -> Created {
    Created::Moment(
        NaiveDate::from_ymd_opt(2026, month, day)
            .unwrap()
            .and_hms_opt(hour, minute, 0)
            .unwrap(),
    )
}

fn now() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 11, 4)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap()
}

#[test]
fn short_id_drops_the_date_and_the_handoff_prefix() {
    for (id, short) in [
        ("S46-owner-gates-2026-11-04", "S46-owner-gates"),
        ("HANDOFF-2026-11-04-next-session", "next-session"),
        ("S-2026-11-04-fa340b82", "S-fa340b82"),
        ("S38-next", "S38-next"),
        ("2026-11-04", "2026-11-04"),
        ("HANDOFF-2026-11-04", "HANDOFF"),
        ("", ""),
    ] {
        assert_eq!(short_id(id), short, "{id:?}");
    }
}

#[test]
fn age_is_compact_and_relative() {
    let cases = [
        ("2026-11-04T12:00:00", "now"),
        ("2026-11-04T11:59:30", "now"),
        ("2026-11-04T11:55:00", "5m"),
        ("2026-11-04T09:00:00", "3h"),
        ("2026-11-02T12:00:00", "2d"),
        ("2026-11-02T20:00:00", "1d"),
        ("2026-11-04T12:30:00", "now"),
        ("2026-11-04", "today"),
        ("2026-11-02", "2d"),
        ("2026-11-05", "today"),
    ];
    for (created, shown) in cases {
        assert_eq!(age(created, now()).as_deref(), Some(shown), "{created:?}");
    }
    for created in ["", "garbage", "2026-13-01"] {
        assert_eq!(age(created, now()), None, "{created:?}");
    }
}

#[test]
fn an_active_row_line_is_the_short_id_then_the_age() {
    let handoff = fixture(
        "S46-owner-gates-2026-11-04",
        "rental-app",
        "2026-11-04T09:00:00",
        "open",
    );
    assert_eq!(detail(&handoff, None, now()), "S46-owner-gates \u{b7} 3h");
}

#[test]
fn a_row_without_a_usable_created_shows_the_short_id_only() {
    for created in ["", "not a date", "2026-13-01"] {
        let handoff = fixture("S38-next", "kb", created, "open");
        assert_eq!(detail(&handoff, None, now()), "S38-next", "{created:?}");
    }
}

#[test]
fn a_history_row_ends_with_why_it_is_there() {
    let dated = fixture("S1", "remora", "2026-11-04T11:00:00", "open");
    assert_eq!(
        detail(&dated, Some(Shelf::PickedUp), now()),
        "S1 \u{b7} 1h \u{b7} picked up"
    );
    assert_eq!(
        detail(&dated, Some(Shelf::Superseded), now()),
        "S1 \u{b7} 1h \u{b7} superseded"
    );
    assert_eq!(
        detail(&dated, Some(Shelf::Dismissed), now()),
        "S1 \u{b7} 1h \u{b7} dismissed"
    );
    let undated = fixture("S1", "kb", "not a date", "claimed");
    assert_eq!(
        detail(&undated, Some(Shelf::PickedUp), now()),
        "S1 \u{b7} picked up"
    );
}
