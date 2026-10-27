//! The Devices layout's one list: under each host, agents grouped by the
//! worktree they run in. This module is the pure part: which rows exist, in
//! what groups, and what text they carry. Painting lives in `render`.

use herdr_client::protocol::AgentStatus;
use std::collections::HashSet;

pub(super) mod facts;

/// What the list needs to know about one agent, resolved once from the
/// snapshot so grouping, text, and filtering never touch GPUI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Facts {
    pub status: AgentStatus,
    pub title: String,
    pub workspace: Option<String>,
    pub branch: Option<String>,
    pub model: Option<String>,
    /// Agents with the same key share a worktree; it includes the host.
    pub group_key: String,
    pub group_label: String,
}

/// A worktree's sessions under one collapsible header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Group {
    pub key: String,
    pub label: String,
    pub branch: Option<String>,
    /// Indexes into the facts, in list order.
    pub members: Vec<usize>,
    /// Statuses present with their counts, most urgent first.
    pub summary: Vec<(AgentStatus, usize)>,
    pub collapsed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Entry {
    /// A worktree with one session: a flat row.
    Row(usize),
    Group(Group),
}

/// The second line of a row: workspace, branch, model, each omitted when
/// the daemon did not report it.
pub(super) fn detail_line(facts: &Facts) -> Option<String> {
    let parts: Vec<&str> = [&facts.workspace, &facts.branch, &facts.model]
        .into_iter()
        .filter_map(|part| part.as_deref())
        .filter(|part| !part.is_empty())
        .collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// Whether every word of `query` is in the row's title, workspace, branch,
/// or worktree name. A blank query matches everything.
pub(super) fn matches(facts: &Facts, query: &str) -> bool {
    let fields = [
        Some(facts.title.as_str()),
        facts.workspace.as_deref(),
        facts.branch.as_deref(),
        Some(facts.group_label.as_str()),
    ]
    .map(|field| field.unwrap_or_default().to_lowercase());
    words(query).all(|word| fields.iter().any(|field| field.contains(&word)))
}

/// Whether every word of `query` is in a host's name.
pub(super) fn host_matches(label: &str, query: &str) -> bool {
    let label = label.to_lowercase();
    words(query).all(|word| label.contains(&word))
}

fn words(query: &str) -> impl Iterator<Item = String> + '_ {
    query.split_whitespace().map(str::to_lowercase)
}

/// The entries to paint for one host's agents. `host_named` keeps every agent
/// when the query names the host itself. A group sits where its first member
/// was and its members stay in list order.
pub(super) fn arrange(
    facts: &[Facts],
    query: &str,
    host_named: bool,
    collapsed: &HashSet<String>,
) -> Vec<Entry> {
    let kept: Vec<usize> = (0..facts.len())
        .filter(|&index| host_named || matches(&facts[index], query))
        .collect();
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    for &index in &kept {
        let key = &facts[index].group_key;
        if !seen.insert(key) {
            continue;
        }
        let members: Vec<usize> = kept
            .iter()
            .copied()
            .filter(|&other| facts[other].group_key == *key)
            .collect();
        if let [only] = members.as_slice() {
            entries.push(Entry::Row(*only));
            continue;
        }
        let mut summary: Vec<(AgentStatus, usize)> = Vec::new();
        for &member in &members {
            let status = facts[member].status;
            match summary.iter_mut().find(|(seen, _)| *seen == status) {
                Some((_, count)) => *count += 1,
                None => summary.push((status, 1)),
            }
        }
        summary
            .sort_by_key(|(status, _)| std::cmp::Reverse(super::agents::status_priority(*status)));
        entries.push(Entry::Group(Group {
            key: key.clone(),
            label: facts[index].group_label.clone(),
            branch: facts[index].branch.clone(),
            members,
            summary,
            collapsed: collapsed.contains(key),
        }));
    }
    entries
}

/// Folds `key`, or unfolds it when folded.
pub(super) fn toggle(collapsed: &mut HashSet<String>, key: &str) {
    if !collapsed.remove(key) {
        collapsed.insert(key.to_owned());
    }
}

#[cfg(test)]
mod tests;
