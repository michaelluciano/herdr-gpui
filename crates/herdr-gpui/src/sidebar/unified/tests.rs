use super::*;
use AgentStatus::{Blocked, Idle, Working};

fn fact(title: &str, status: AgentStatus, key: &str) -> Facts {
    Facts {
        status,
        title: title.into(),
        workspace: Some("herdr".into()),
        branch: Some("main".into()),
        model: None,
        group_key: key.into(),
        group_label: key.into(),
    }
}

fn none() -> HashSet<String> {
    HashSet::new()
}

#[test]
fn sessions_sharing_a_worktree_become_one_group_with_count_and_states() {
    let facts = [
        fact("a", Working, "wt1"),
        fact("b", Blocked, "wt1"),
        fact("c", Working, "wt1"),
    ];
    let entries = arrange(&facts, "", false, &none());
    let [Entry::Group(group)] = entries.as_slice() else {
        panic!("one group expected, got {entries:?}");
    };
    assert_eq!(group.members, vec![0, 1, 2]);
    // Most urgent first.
    assert_eq!(group.summary, vec![(Blocked, 1), (Working, 2)]);
    assert!(!group.collapsed);
}

#[test]
fn a_lone_session_stays_a_flat_row() {
    let facts = [fact("a", Idle, "wt1"), fact("b", Idle, "wt2")];
    assert_eq!(
        arrange(&facts, "", false, &none()),
        vec![Entry::Row(0), Entry::Row(1)]
    );
}

#[test]
fn groups_sit_at_their_first_member_and_order_is_stable() {
    let facts = [
        fact("a", Idle, "wt1"),
        fact("b", Idle, "solo"),
        fact("c", Idle, "wt1"),
    ];
    let entries = arrange(&facts, "", false, &none());
    assert!(matches!(&entries[0], Entry::Group(group) if group.members == vec![0, 2]));
    assert_eq!(entries[1], Entry::Row(1));
    assert_eq!(entries.len(), 2);
    assert_eq!(entries, arrange(&facts, "", false, &none()));
}

#[test]
fn the_group_carries_the_first_members_branch_and_label() {
    let mut first = fact("a", Idle, "wt1");
    first.branch = Some("feat/x".into());
    first.group_label = "herdr-x".into();
    let facts = [first, fact("b", Idle, "wt1")];
    let entries = arrange(&facts, "", false, &none());
    let [Entry::Group(group)] = entries.as_slice() else {
        panic!("one group expected");
    };
    assert_eq!(group.label, "herdr-x");
    assert_eq!(group.branch.as_deref(), Some("feat/x"));
}

#[test]
fn collapse_state_toggles_and_marks_the_group() {
    let facts = [fact("a", Idle, "wt1"), fact("b", Idle, "wt1")];
    let mut collapsed = none();
    toggle(&mut collapsed, "wt1");
    let entries = arrange(&facts, "", false, &collapsed);
    assert!(matches!(&entries[0], Entry::Group(group) if group.collapsed));
    toggle(&mut collapsed, "wt1");
    assert!(collapsed.is_empty());
    let entries = arrange(&facts, "", false, &collapsed);
    assert!(matches!(&entries[0], Entry::Group(group) if !group.collapsed));
}

#[test]
fn detail_line_joins_present_fields_and_omits_missing_ones() {
    let mut row = fact("t", Idle, "k");
    row.model = Some("opus".into());
    assert_eq!(detail_line(&row).as_deref(), Some("herdr · main · opus"));
    row.branch = None;
    assert_eq!(detail_line(&row).as_deref(), Some("herdr · opus"));
    row.workspace = None;
    row.model = None;
    assert_eq!(detail_line(&row), None);
}

#[test]
fn the_filter_matches_title_workspace_and_branch_by_word() {
    let mut row = fact("Fix login bug", Idle, "k");
    row.branch = Some("feat/auth".into());
    assert!(matches(&row, ""));
    assert!(matches(&row, "LOGIN"));
    assert!(matches(&row, "herdr"));
    assert!(matches(&row, "auth login"));
    assert!(!matches(&row, "auth missing"));
}

#[test]
fn the_filter_keeps_a_group_header_only_while_a_child_matches() {
    let mut other = fact("other", Idle, "wt1");
    other.workspace = Some("zzz".into());
    other.branch = Some("zzz".into());
    let mut hit = fact("login", Idle, "wt1");
    hit.workspace = Some("zzz".into());
    hit.branch = Some("zzz".into());
    let mut hit2 = hit.clone();
    hit2.title = "login again".into();
    let facts = [other.clone(), hit, hit2];
    let entries = arrange(&facts, "login", false, &none());
    let [Entry::Group(group)] = entries.as_slice() else {
        panic!("group expected, got {entries:?}");
    };
    assert_eq!(group.members, vec![1, 2]);
    // One survivor is a flat row; no match leaves nothing at all.
    let entries = arrange(&[other.clone(), facts[1].clone()], "login", false, &none());
    assert_eq!(entries, vec![Entry::Row(1)]);
    assert!(arrange(&[other], "login", false, &none()).is_empty());
}

#[test]
fn naming_the_host_keeps_all_its_agents() {
    let facts = [fact("a", Idle, "wt1")];
    assert!(arrange(&facts, "build", false, &none()).is_empty());
    assert_eq!(arrange(&facts, "build", true, &none()), vec![Entry::Row(0)]);
    assert!(host_matches("Build box", "build BOX"));
    assert!(!host_matches("Build box", "login"));
}
