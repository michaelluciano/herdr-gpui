//! What is running now: the llama servers in `ps`, and each row's health.
//! The probes block; only a background task calls them.
use super::state::{Health, RowInput, RowState, health_from_status, row_states};
use crate::config::llm_launchers::{Kind, Launcher, canonical_host};
use std::io::Read;
use std::process::Stdio;
use std::time::Duration;

/// Most bytes of `ps` output read or parsed.
pub(crate) const PS_BYTES: usize = 1 << 20;
/// Most llama server ports reported.
pub(crate) const MAX_PORTS: usize = 32;
/// How long a health probe waits in the background poll.
pub(crate) const PROBE_TIMEOUT: Duration = Duration::from_secs(1);

/// What the last poll saw.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Observed {
    /// The probe of `launchers[i]`; `None` for terminal rows and rows on
    /// another machine.
    pub health: Vec<Option<Health>>,
    /// Local llama servers that no row names.
    pub foreign: Vec<u16>,
}

/// The ports of the `llama-server` processes in `ps -axo pid=,command=`.
pub(crate) fn parse_llama_ports(ps_output: &str) -> Vec<u16> {
    let mut text = ps_output;
    if text.len() > PS_BYTES {
        let mut end = PS_BYTES;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        // The cut line may end mid-number.
        end = text[..end].rfind('\n').unwrap_or(0);
        text = &text[..end];
    }
    let mut ports = Vec::new();
    for line in text.lines() {
        let Some(port) = llama_port(line) else {
            continue;
        };
        if ports.len() >= MAX_PORTS {
            break;
        }
        if !ports.contains(&port) {
            ports.push(port);
        }
    }
    ports
}

/// The `--port` of a `ps` line whose command is `llama-server`.
fn llama_port(line: &str) -> Option<u16> {
    let mut tokens = line.split_whitespace().skip(1);
    let command = tokens.next()?;
    if command.rsplit('/').next() != Some("llama-server") {
        return None;
    }
    let mut args = tokens;
    while let Some(arg) = args.next() {
        let value = if arg == "--port" {
            args.next()?
        } else if let Some(value) = arg.strip_prefix("--port=") {
            value
        } else {
            continue;
        };
        return parse_port(value);
    }
    None
}

fn parse_port(value: &str) -> Option<u16> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok().filter(|port| *port != 0)
}

/// Running ports that no configured row names, in order.
pub(crate) fn foreign_ports(running: &[u16], named: &[u16]) -> Vec<u16> {
    running
        .iter()
        .copied()
        .filter(|port| !named.contains(port))
        .collect()
}

/// The port of a llama row on this machine.
fn local_llama_port(launcher: &Launcher) -> Option<u16> {
    match launcher.kind {
        Kind::Llama { port, .. } if canonical_host(&launcher.host) == "local" => Some(port),
        _ => None,
    }
}

/// Whether any row is a llama row on this machine, so `ps` is worth running.
pub(crate) fn needs_ps(launchers: &[Launcher]) -> bool {
    launchers.iter().any(|row| local_llama_port(row).is_some())
}

/// Assembles the observation from `ps` output, probing each local llama row.
pub(crate) fn observe_with(
    launchers: &[Launcher],
    ps_output: &str,
    probe: impl Fn(u16) -> Health,
) -> Observed {
    let health: Vec<Option<Health>> = launchers
        .iter()
        .map(|row| local_llama_port(row).map(&probe))
        .collect();
    let named: Vec<u16> = launchers.iter().filter_map(local_llama_port).collect();
    Observed {
        health,
        foreign: foreign_ports(&parse_llama_ports(ps_output), &named),
    }
}

/// The state of every row; an observation of a different row count is padded
/// with `None` or trimmed.
pub(crate) fn states(launchers: &[Launcher], observed: &Observed) -> Vec<RowState> {
    let rows: Vec<RowInput> = launchers
        .iter()
        .enumerate()
        .map(|(index, row)| RowInput {
            host: &row.host,
            group: row.group.as_deref(),
            health: observed.health.get(index).copied().flatten(),
        })
        .collect();
    let foreign: Vec<(&str, u16)> = observed
        .foreign
        .iter()
        .map(|port| ("Local", *port))
        .collect();
    row_states(&rows, &foreign)
}

/// GET `/health` of a llama server on this machine. Blocking.
pub(crate) fn probe_health(port: u16, timeout: Duration) -> Health {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .max_redirects(0)
        .build()
        .into();
    match agent.get(format!("http://127.0.0.1:{port}/health")).call() {
        Ok(response) => health_from_status(Some(response.status().as_u16())),
        Err(ureq::Error::StatusCode(status)) => health_from_status(Some(status)),
        Err(_) => Health::Down,
    }
}

/// `ps -axo pid=,command=`, bounded; empty when it cannot start. Blocking.
pub(crate) fn run_ps() -> String {
    let mut command = herdr_client::new_command("ps");
    command
        .args(["-axo", "pid=,command="])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    crate::login_env::apply(&mut command);
    let Ok(mut child) = command.spawn() else {
        return String::new();
    };
    let mut bytes = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let mut limited = stdout.take(PS_BYTES as u64);
        if limited.read_to_end(&mut bytes).is_err() {
            bytes.clear();
        }
    }
    // Closed stdout ends a ps that had more to say; kill covers one that hangs.
    child.kill().ok();
    child.wait().ok();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Looks at the machine. Blocking.
pub(crate) fn observe(launchers: &[Launcher]) -> Observed {
    let ps = if needs_ps(launchers) {
        run_ps()
    } else {
        String::new()
    };
    observe_with(launchers, &ps, |port| probe_health(port, PROBE_TIMEOUT))
}
