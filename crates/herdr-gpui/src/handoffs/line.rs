//! A handoff row's second line: its project, when it was written, and why a
//! history row is there.

use super::{
    classify::Shelf,
    list::{Handoff, Kind},
};

/// `MM-DD HH:MM` from a local ISO `created` such as `2026-10-31T16:37:41`
/// (seconds optional), or `MM-DD` from a date alone such as `2026-10-31`.
/// `None` for anything else.
pub(crate) fn short_stamp(created: &str) -> Option<String> {
    let Some((date, time)) = created.split_once('T') else {
        let (month, day) = month_day(created)?;
        return Some(format!("{month}-{day}"));
    };
    let (month, day) = month_day(date)?;
    let mut clock = time.split(':');
    let (hour, minute) = (clock.next()?, clock.next()?);
    let second = clock.next();
    if clock.next().is_some() {
        return None;
    }
    field(hour, 0, 23)?;
    field(minute, 0, 59)?;
    if let Some(second) = second {
        field(second, 0, 59)?;
    }
    Some(format!("{month}-{day} {hour}:{minute}"))
}

/// The `(month, day)` of a `YYYY-MM-DD` date, both in range.
fn month_day(date: &str) -> Option<(&str, &str)> {
    let mut parts = date.split('-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || year.len() != 4 || !all_digits(year) {
        return None;
    }
    field(month, 1, 12)?;
    field(day, 1, 31)?;
    Some((month, day))
}

/// Exactly two ASCII digits within `min..=max`.
fn field(value: &str, min: u32, max: u32) -> Option<u32> {
    if value.len() != 2 || !all_digits(value) {
        return None;
    }
    value.parse().ok().filter(|n| (min..=max).contains(n))
}

fn all_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

/// The first line of a row: the project for a `LATEST.md` handoff, else the id.
pub(crate) fn title(handoff: &Handoff) -> &str {
    match handoff.kind {
        Kind::Latest => &handoff.project,
        Kind::History | Kind::Legacy => &handoff.id,
    }
}

/// The second line of a row. A `LATEST.md` row shows the stamp when
/// `created` has one, then its summary; any other row shows the project,
/// then the stamp. A history row ends with the reason it is there.
pub(crate) fn detail(handoff: &Handoff, shelf: Option<Shelf>) -> String {
    let stamp = short_stamp(&handoff.created);
    let mut parts: Vec<&str> = Vec::new();
    if handoff.kind == Kind::Latest {
        parts.extend(stamp.as_deref());
        if !handoff.summary.is_empty() {
            parts.push(&handoff.summary);
        }
    } else {
        parts.push(&handoff.project);
        parts.extend(stamp.as_deref());
    }
    let mut line = parts.join(" \u{b7} ");
    if let Some(shelf) = shelf {
        line.push_str(" \u{b7} ");
        line.push_str(shelf.label());
    }
    line
}
