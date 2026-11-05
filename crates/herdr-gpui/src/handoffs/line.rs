//! A handoff row's second line: its short id, how long ago it was written, and
//! why a history row is there.

use super::{classify::Shelf, list::Handoff};
use chrono::{NaiveDate, NaiveDateTime};

/// When a handoff was written: a local date and time, or a date alone.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Created {
    Day(NaiveDate),
    Moment(NaiveDateTime),
}

/// Reads a local ISO `created` such as `2026-10-31T16:37:41` (seconds
/// optional) or a date alone such as `2026-10-31`. `None` for anything else.
pub(crate) fn parse_created(created: &str) -> Option<Created> {
    let Some((date, time)) = created.split_once('T') else {
        return Some(Created::Day(date_of(created)?));
    };
    let date = date_of(date)?;
    let mut clock = time.split(':');
    let (hour, minute) = (clock.next()?, clock.next()?);
    let second = clock.next();
    if clock.next().is_some() {
        return None;
    }
    let hour = field(hour, 0, 23)?;
    let minute = field(minute, 0, 59)?;
    let second = match second {
        Some(second) => field(second, 0, 59)?,
        None => 0,
    };
    Some(Created::Moment(date.and_hms_opt(hour, minute, second)?))
}

/// A `YYYY-MM-DD` date that exists.
fn date_of(date: &str) -> Option<NaiveDate> {
    let mut parts = date.split('-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || year.len() != 4 || !all_digits(year) {
        return None;
    }
    let month = field(month, 1, 12)?;
    let day = field(day, 1, 31)?;
    NaiveDate::from_ymd_opt(year.parse().ok()?, month, day)
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

/// The id without its dates and `HANDOFF-` prefix: `S-2026-11-04-fa340b82`
/// reads `S-fa340b82`. An id that would be left empty stays whole.
pub(crate) fn short_id(id: &str) -> String {
    let mut bare = String::new();
    let bytes = id.as_bytes();
    let mut at = 0;
    while at < id.len() {
        if is_date_at(bytes, at) {
            at += 10;
        } else {
            let ch = id[at..].chars().next().unwrap_or_default();
            bare.push(ch);
            at += ch.len_utf8().max(1);
        }
    }
    let bare = tidy(&bare);
    let bare = tidy(bare.strip_prefix("HANDOFF-").unwrap_or(&bare));
    if bare.is_empty() {
        id.to_string()
    } else {
        bare
    }
}

/// Whether `YYYY-MM-DD` (digits only) starts at `at`.
fn is_date_at(bytes: &[u8], at: usize) -> bool {
    bytes.get(at..at + 10).is_some_and(|date| {
        date.iter().enumerate().all(|(i, byte)| match i {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
    })
}

/// Collapses runs of `-` and trims them from both ends.
fn tidy(text: &str) -> String {
    text.split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// How long ago `created` was, compactly: `now`, `5m`, `3h`, `2d`. A date
/// alone resolves to days only: `today` or `Nd`. `None` when `created` does
/// not parse. A time ahead of `now` reads as now.
pub(crate) fn age(created: &str, now: NaiveDateTime) -> Option<String> {
    match parse_created(created)? {
        Created::Day(day) => {
            let days = (now.date() - day).num_days();
            Some(if days <= 0 {
                "today".into()
            } else {
                format!("{days}d")
            })
        }
        Created::Moment(then) => {
            let minutes = (now - then).num_minutes();
            Some(if minutes < 1 {
                "now".into()
            } else if minutes < 60 {
                format!("{minutes}m")
            } else if minutes < 60 * 24 {
                format!("{}h", minutes / 60)
            } else {
                format!("{}d", minutes / (60 * 24))
            })
        }
    }
}

/// The second line of a row: the short id, then the age when `created`
/// parses, then the reason a history row is there.
pub(crate) fn detail(handoff: &Handoff, shelf: Option<Shelf>, now: NaiveDateTime) -> String {
    let mut line = short_id(&handoff.id);
    if let Some(age) = age(&handoff.created, now) {
        line.push_str(" \u{b7} ");
        line.push_str(&age);
    }
    if let Some(shelf) = shelf {
        line.push_str(" \u{b7} ");
        line.push_str(shelf.label());
    }
    line
}
