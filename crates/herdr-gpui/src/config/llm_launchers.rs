//! `[[llm_launchers]]`: sidebar rows under a host that start a local model
//! server or run a terminal command.
use serde::Deserialize;

/// Rows kept; extra ones are dropped and named like any ignored key.
pub(crate) const MAX_ROWS: usize = 16;
/// Longest label, in bytes.
pub(crate) const MAX_LABEL_BYTES: usize = 80;
/// Longest `start` or `run` command, in bytes.
pub(crate) const MAX_COMMAND_BYTES: usize = 512;

/// Why a launcher row was refused.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("llm_launchers.{row}.{field} must not contain control characters")]
    ControlChar { row: usize, field: &'static str },
    #[error("llm_launchers.{row}.{field} must not be empty")]
    Empty { row: usize, field: &'static str },
    #[error("llm_launchers.{row}.port must be 1 to 65535")]
    InvalidPort { row: usize },
    #[error("llm_launchers.{row}.{field} is required")]
    MissingField { row: usize, field: &'static str },
    #[error("llm_launchers.{row}.{field} is at most {max} bytes")]
    TooLong {
        row: usize,
        field: &'static str,
        max: usize,
    },
    #[error("llm_launchers.{row}.kind must be \"llama\" or \"terminal\", not {kind:?}")]
    UnknownKind { row: usize, kind: String },
    #[error("llm_launchers.{row}.{field} does not belong on a {kind} row")]
    WrongField {
        row: usize,
        kind: &'static str,
        field: &'static str,
    },
    // keep-sorted end
}

/// What a row starts or runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A llama server: `start` brings it up on `port`, and `url` (default
    /// `http://127.0.0.1:PORT/`) is what opens once it answers.
    Llama {
        port: u16,
        start: String,
        url: Option<String>,
    },
    /// A command typed into a terminal on the host.
    Terminal { run: String },
}

/// One resolved row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launcher {
    pub label: String,
    /// An endpoint's label or id.
    pub host: String,
    /// Rows of one host and group are mutually exclusive.
    pub group: Option<String>,
    pub kind: Kind,
}

impl Launcher {
    /// Whether this row's `host` names the endpoint, by label or id. The
    /// match is exact and case-sensitive, except that `local` and `Local`
    /// are one name.
    pub fn on_host(&self, endpoint_label: &str, endpoint_id: &str) -> bool {
        let host = canonical_host(&self.host);
        host == canonical_host(endpoint_label) || host == canonical_host(endpoint_id)
    }
}

/// The name a host is compared under: the local machine is written both
/// `Local` (its label) and `local` (its id).
pub(crate) fn canonical_host(host: &str) -> &str {
    if host == "Local" { "local" } else { host }
}

/// A row as written.
#[derive(Default, Deserialize)]
#[serde(default)]
pub(crate) struct Raw {
    label: Option<String>,
    host: Option<String>,
    kind: Option<String>,
    group: Option<String>,
    port: Option<i64>,
    start: Option<String>,
    url: Option<String>,
    run: Option<String>,
}

/// Drops rows past [`MAX_ROWS`], returning each dropped row's key.
pub(crate) fn limit(rows: &mut Vec<Raw>) -> Vec<String> {
    let dropped = rows.len().saturating_sub(MAX_ROWS);
    rows.truncate(MAX_ROWS);
    (MAX_ROWS..MAX_ROWS + dropped)
        .map(|row| format!("llm_launchers.{row}"))
        .collect()
}

/// Checks every row and builds the launchers, failing on the first bad one.
pub(crate) fn resolve(rows: Vec<Raw>) -> Result<Vec<Launcher>, Error> {
    rows.into_iter()
        .enumerate()
        .map(|(row, raw)| resolve_row(row, raw))
        .collect()
}

fn text(
    row: usize,
    field: &'static str,
    value: Option<String>,
    max: usize,
) -> Result<String, Error> {
    let value = value.ok_or(Error::MissingField { row, field })?;
    if value.is_empty() {
        return Err(Error::Empty { row, field });
    }
    if value.len() > max {
        return Err(Error::TooLong { row, field, max });
    }
    if value.chars().any(char::is_control) {
        return Err(Error::ControlChar { row, field });
    }
    Ok(value)
}

fn forbid<T>(
    row: usize,
    kind: &'static str,
    field: &'static str,
    value: &Option<T>,
) -> Result<(), Error> {
    if value.is_some() {
        return Err(Error::WrongField { row, kind, field });
    }
    Ok(())
}

fn resolve_row(row: usize, raw: Raw) -> Result<Launcher, Error> {
    let label = text(row, "label", raw.label, MAX_LABEL_BYTES)?;
    let host = text(row, "host", raw.host, MAX_LABEL_BYTES)?;
    let group = raw
        .group
        .map(|group| text(row, "group", Some(group), MAX_LABEL_BYTES))
        .transpose()?;
    let kind = match raw.kind.as_deref() {
        None => return Err(Error::MissingField { row, field: "kind" }),
        Some("llama") => {
            forbid(row, "llama", "run", &raw.run)?;
            let port = raw.port.ok_or(Error::MissingField { row, field: "port" })?;
            let port = u16::try_from(port)
                .ok()
                .filter(|port| *port != 0)
                .ok_or(Error::InvalidPort { row })?;
            let start = text(row, "start", raw.start, MAX_COMMAND_BYTES)?;
            Kind::Llama {
                port,
                start,
                url: raw.url,
            }
        }
        Some("terminal") => {
            forbid(row, "terminal", "port", &raw.port)?;
            forbid(row, "terminal", "start", &raw.start)?;
            forbid(row, "terminal", "url", &raw.url)?;
            Kind::Terminal {
                run: text(row, "run", raw.run, MAX_COMMAND_BYTES)?,
            }
        }
        Some(other) => {
            return Err(Error::UnknownKind {
                row,
                kind: other.to_owned(),
            });
        }
    };
    Ok(Launcher {
        label,
        host,
        group,
        kind,
    })
}
