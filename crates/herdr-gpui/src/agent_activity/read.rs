//! Reads the per-pane state files Claude Code's hooks write.

use super::label::{Report, ReportState};
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
};

/// A file longer than this is not a report.
const MAX_BYTES: u64 = 2048;
/// The longest session directory kept, in bytes.
const CWD_LIMIT: usize = 1024;
/// The longest tool name shown.
const TOOL_LIMIT: usize = 32;

/// The state directory under a home directory.
pub(crate) fn dir_under(home: &Path) -> PathBuf {
    home.join(".local").join("state").join("herdr-claude")
}

/// The directory under `$HOME`, `None` without one.
pub(crate) fn state_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(|home| dir_under(Path::new(&home)))
}

/// A pane id as a file stem: anything outside `[A-Za-z0-9_.-]` becomes `_`.
pub(crate) fn file_stem(pane_id: &str) -> String {
    pane_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Parses a report; None for anything not exactly the contract.
pub(crate) fn parse(text: &str) -> Option<Report> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let state = match value.get("state")?.as_str()? {
        "thinking" => ReportState::Thinking,
        "working" => ReportState::Working,
        "subagents" => ReportState::Subagents,
        "waiting" => ReportState::Waiting,
        _ => return None,
    };
    let since = value.get("since")?.as_u64()?;
    let subagents = value
        .get("subagents")
        .and_then(serde_json::Value::as_u64)
        .map_or(0, |n| u32::try_from(n).unwrap_or(u32::MAX));
    let tool = value
        .get("tool")
        .and_then(serde_json::Value::as_str)
        .map(|tool| {
            crate::notifications::safe_text(tool, TOOL_LIMIT)
                .trim()
                .to_owned()
        })
        .unwrap_or_default();
    let cwd = value
        .get("cwd")
        .and_then(serde_json::Value::as_str)
        .filter(|cwd| {
            cwd.len() <= CWD_LIMIT
                && Path::new(cwd).is_absolute()
                && !cwd.chars().any(char::is_control)
        })
        .map(str::to_owned);
    Some(Report {
        state,
        since,
        subagents,
        tool,
        cwd,
    })
}

/// One pane's report from `dir`; a missing, unreadable, oversize or
/// unparsable file is no report.
pub(crate) fn read_report(dir: &Path, pane_id: &str) -> Option<Report> {
    let file = std::fs::File::open(dir.join(format!("{}.json", file_stem(pane_id)))).ok()?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    parse(std::str::from_utf8(&bytes).ok()?)
}

/// The reports of `pane_ids` that have one, keyed by pane id.
pub(crate) fn read_reports(dir: &Path, pane_ids: &[String]) -> HashMap<String, Report> {
    pane_ids
        .iter()
        .filter_map(|id| Some((id.clone(), read_report(dir, id)?)))
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("herdr-activity-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_directory_is_under_local_state() {
        assert_eq!(
            dir_under(Path::new("/h")),
            Path::new("/h/.local/state/herdr-claude")
        );
    }

    #[test]
    fn pane_ids_become_safe_file_stems() {
        assert_eq!(file_stem("w1:p2/../x y"), "w1_p2_.._x_y");
        assert_eq!(file_stem("ok_1.2-3"), "ok_1.2-3");
    }

    #[test]
    fn a_report_parses_and_unknown_keys_are_ignored() {
        let report =
            parse(r#"{"state":"working","since":42,"subagents":2,"tool":"Bash","extra":[1]}"#)
                .unwrap();
        assert_eq!(report.state, ReportState::Working);
        assert_eq!(
            (report.since, report.subagents, report.tool.as_str()),
            (42, 2, "Bash")
        );
        let bare = parse(r#"{"state":"waiting","since":1}"#).unwrap();
        assert_eq!((bare.subagents, bare.tool.as_str()), (0, ""));
    }

    #[test]
    fn the_session_directory_is_kept_only_when_absolute_and_clean() {
        let cwd = |value: &str| {
            parse(&format!(
                r#"{{"state":"waiting","since":1,"cwd":{}}}"#,
                serde_json::to_string(value).unwrap()
            ))
            .unwrap()
            .cwd
        };
        assert_eq!(cwd("/Users/me/kb"), Some("/Users/me/kb".into()));
        assert_eq!(cwd("/a \"q\"/b\\c"), Some("/a \"q\"/b\\c".into()));
        for bad in ["", "relative/dir", "/a/b\nc", "/a/\u{7}b"] {
            assert_eq!(cwd(bad), None, "{bad:?}");
        }
        assert_eq!(cwd(&format!("/{}", "a".repeat(1024))), None);
        assert_eq!(
            parse(r#"{"state":"waiting","since":1,"cwd":7}"#)
                .unwrap()
                .cwd,
            None
        );
        assert_eq!(parse(r#"{"state":"waiting","since":1}"#).unwrap().cwd, None);
    }

    #[test]
    fn bad_reports_are_none() {
        for text in [
            "",
            "not json",
            "[]",
            r#"{"state":"sleeping","since":1}"#,
            r#"{"state":"working"}"#,
            r#"{"state":"working","since":-1}"#,
            r#"{"state":"working","since":"1"}"#,
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }

    #[test]
    fn the_tool_name_is_bounded_and_stripped() {
        let long = "x".repeat(100);
        let report = parse(&format!(
            r#"{{"state":"working","since":1,"tool":"\u001b[31m{long}"}}"#
        ))
        .unwrap();
        assert!(report.tool.chars().count() <= TOOL_LIMIT);
        assert!(!report.tool.contains('\u{1b}'));
    }

    #[test]
    fn files_are_read_by_sanitized_pane_id_and_bounded() {
        let dir = temp("files");
        std::fs::write(
            dir.join("w1_p2.json"),
            r#"{"state":"thinking","since":7,"subagents":0,"tool":""}"#,
        )
        .unwrap();
        let ok = read_report(&dir, "w1:p2").unwrap();
        assert_eq!((ok.state, ok.since), (ReportState::Thinking, 7));
        assert_eq!(read_report(&dir, "missing"), None);

        let padding = " ".repeat(2048);
        std::fs::write(
            dir.join("big.json"),
            format!(r#"{{"state":"thinking","since":7}}{padding}"#),
        )
        .unwrap();
        assert_eq!(read_report(&dir, "big"), None);
        std::fs::write(dir.join("bin.json"), [0xff, 0xfe]).unwrap();
        assert_eq!(read_report(&dir, "bin"), None);

        let all = read_reports(&dir, &["w1:p2".into(), "big".into(), "nope".into()]);
        assert_eq!(all.len(), 1);
        assert!(all.contains_key("w1:p2"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
