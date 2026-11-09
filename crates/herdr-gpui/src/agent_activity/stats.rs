//! What a session row shows of its Claude transcript: model letter and context size.

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextBand {
    Normal,
    Warm,
    Alert,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Stats {
    pub model: String,
    pub context: u64,
}

/// The token count floored to a 5k step as `115k`, or `<5k` below the first step.
pub(crate) fn format_context(tokens: u64) -> String {
    if tokens < 5_000 {
        "<5k".to_string()
    } else {
        let step = (tokens / 5_000) * 5_000;
        format!("{}k", step / 1_000)
    }
}

/// Determines the context band based on usage and an optional limit.
/// If the limit is `None` or zero, returns `Normal`.
/// If usage is >= 90 percent of the limit, returns `Alert`.
/// If usage is >= 70 percent of the limit, returns `Warm`.
/// Otherwise returns `Normal`.
pub(crate) fn context_band(used: u64, limit: Option<u64>) -> ContextBand {
    match limit {
        None | Some(0) => ContextBand::Normal,
        Some(lim) => {
            let used_pct = (used as u128) * 100;
            let lim_90 = (lim as u128) * 90;
            let lim_70 = (lim as u128) * 70;
            if used_pct >= lim_90 {
                ContextBand::Alert
            } else if used_pct >= lim_70 {
                ContextBand::Warm
            } else {
                ContextBand::Normal
            }
        }
    }
}

/// Returns a single character representing the model type, if recognizable.
/// Recognizes haiku, sonnet, opus and fable, ignoring case.
/// Returns `None` for unrecognized or empty strings.
pub(crate) fn model_letter(model: &str) -> Option<char> {
    let lower = model.to_ascii_lowercase();
    if lower.contains("haiku") {
        Some('H')
    } else if lower.contains("sonnet") {
        Some('S')
    } else if lower.contains("opus") {
        Some('O')
    } else if lower.contains("fable") {
        Some('F')
    } else {
        None
    }
}

/// Parses the tail of a Claude transcript (JSONL) and returns the most recent
/// qualifying assistant message as `Stats`.
/// A qualifying message is an assistant line with a non-empty, non-synthetic
/// model and a `usage` object. Missing numeric fields are treated as zero.
/// Lines that cannot be parsed as JSON, are not assistant messages, or
/// lack required fields are skipped. The function scans from the end of the
/// input, so the last qualifying line wins.
/// Returns `None` if no qualifying line is found.
pub(crate) fn parse_transcript_tail(tail: &str) -> Option<Stats> {
    for line in tail.lines().rev() {
        let v: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v.get("type").and_then(|t| t.as_str()) != Some("assistant") {
            continue;
        }
        let message = match v.get("message") {
            Some(m) => m,
            None => continue,
        };
        let model = match message.get("model").and_then(|m| m.as_str()) {
            Some(m) => m,
            None => continue,
        };
        if model.is_empty() || model == "<synthetic>" {
            continue;
        }
        let usage = match message.get("usage") {
            Some(u) => u,
            None => continue,
        };
        let input_tokens = usage
            .get("input_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let cache_creation_input_tokens = usage
            .get("cache_creation_input_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let cache_read_input_tokens = usage
            .get("cache_read_input_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let output_tokens = usage
            .get("output_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let context =
            input_tokens + cache_creation_input_tokens + cache_read_input_tokens + output_tokens;
        return Some(Stats {
            model: model.to_string(),
            context,
        });
    }
    None
}

#[cfg(test)]
mod tests;
