//! Parsing of `gh auth status` and the decision of what the sidebar account chip shows.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GhAccount {
    pub name: String,
    pub active: bool,
}

pub(super) fn parse_gh_accounts(status: &str) -> Vec<GhAccount> {
    let mut accounts = Vec::new();
    let mut inside_github = false;
    let mut expecting_active = false;

    for line in status.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Host block start/end
        if line.chars().next().map_or(false, |c| !c.is_whitespace()) && !trimmed.contains(' ') {
            if trimmed == "github.com" {
                inside_github = true;
            } else {
                inside_github = false;
            }
            continue;
        }
        if !inside_github {
            continue;
        }

        // Skip failed login lines
        if trimmed.contains("Failed to log in") {
            continue;
        }

        // Account line
        if let Some(idx) = trimmed.find("Logged in to github.com account ") {
            let rest = &trimmed[idx + "Logged in to github.com account ".len()..];
            let name = if let Some(end) = rest.find(" (") {
                &rest[..end]
            } else {
                rest
            };
            accounts.push(GhAccount {
                name: name.to_string(),
                active: false,
            });
            expecting_active = true;
            continue;
        }

        // Active flag line
        if trimmed.contains("Active account:") {
            if expecting_active {
                let active = trimmed.contains("true");
                if let Some(last) = accounts.last_mut() {
                    last.active = active;
                }
                expecting_active = false;
            }
            continue;
        }
    }

    accounts
}

pub(super) fn next_account(accounts: &[GhAccount]) -> Option<&str> {
    if accounts.len() < 2 {
        return None;
    }
    let active_idx = accounts.iter().position(|a| a.active)?;
    let next_idx = (active_idx + 1) % accounts.len();
    Some(&accounts[next_idx].name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChipTone {
    Match,
    Mismatch,
    Unknown,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ChipView {
    pub label: String,
    pub tone: ChipTone,
}

pub(super) fn chip_view(accounts: &[GhAccount], workspace_owner: Option<&str>) -> ChipView {
    if accounts.is_empty() {
        return ChipView {
            label: "gh: none".into(),
            tone: ChipTone::Missing,
        };
    }
    let active = accounts.iter().find(|a| a.active);
    let active = match active {
        Some(a) => a,
        None => {
            // No active account; treat as missing
            return ChipView {
                label: "gh: none".into(),
                tone: ChipTone::Missing,
            };
        }
    };
    let label = active.name.clone();
    let tone = match workspace_owner {
        Some(owner) if owner.eq_ignore_ascii_case(&active.name) => ChipTone::Match,
        Some(_) => ChipTone::Mismatch,
        None => ChipTone::Unknown,
    };
    ChipView { label, tone }
}

#[cfg(test)]
mod tests;
