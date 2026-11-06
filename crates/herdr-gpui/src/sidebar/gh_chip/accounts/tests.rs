use super::*;

const GITHUB_STATUS: &str = r#"
github.com
  ✓ Logged in to github.com account mlucianoCAE (keyring)
  - Active account: true
  - Git operations protocol: https
  - Token: gho_************************************
  - Token scopes: 'gist', 'read:org', 'repo', 'workflow'

  ✓ Logged in to github.com account michaelluciano (keyring)
  - Active account: false
  - Git operations protocol: https
  - Token: gho_************************************
  - Token scopes: 'gist', 'project', 'read:org', 'repo', 'workflow'
"#;

const GITHUB_STATUS_SWAPPED: &str = r#"
github.com
  ✓ Logged in to github.com account mlucianoCAE (keyring)
  - Active account: false
  - Git operations protocol: https
  - Token: gho_************************************
  - Token scopes: 'gist', 'read:org', 'repo', 'workflow'

  ✓ Logged in to github.com account michaelluciano (keyring)
  - Active account: true
  - Git operations protocol: https
  - Token: gho_************************************
  - Token scopes: 'gist', 'project', 'read:org', 'repo', 'workflow'
"#;

const GITHUB_STATUS_FAILED: &str = r#"
github.com
  X Failed to log in to github.com account foo (keyring)

  ✓ Logged in to github.com account mlucianoCAE (keyring)
  - Active account: true
  - Git operations protocol: https
  - Token: gho_************************************
  - Token scopes: 'gist', 'read:org', 'repo', 'workflow'
"#;

const GITHUB_STATUS_OTHER_HOST: &str = r#"
github.com
  ✓ Logged in to github.com account mlucianoCAE (keyring)
  - Active account: true
  - Git operations protocol: https
  - Token: gho_************************************
  - Token scopes: 'gist', 'read:org', 'repo', 'workflow'

gitlab.com
  ✓ Logged in to gitlab.com account other (keyring)
  - Active account: true
  - Git operations protocol: https
  - Token: glh_************************************
  - Token scopes: 'api', 'read_user'
"#;

#[test]
fn test_parse_gh_accounts_basic() {
    let accounts = parse_gh_accounts(GITHUB_STATUS);
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].name, "mlucianoCAE");
    assert!(accounts[0].active);
    assert_eq!(accounts[1].name, "michaelluciano");
    assert!(!accounts[1].active);
}

#[test]
fn test_parse_gh_accounts_swapped() {
    let accounts = parse_gh_accounts(GITHUB_STATUS_SWAPPED);
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].name, "mlucianoCAE");
    assert!(!accounts[0].active);
    assert_eq!(accounts[1].name, "michaelluciano");
    assert!(accounts[1].active);
}

#[test]
fn test_parse_gh_accounts_failed_login() {
    let accounts = parse_gh_accounts(GITHUB_STATUS_FAILED);
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].name, "mlucianoCAE");
    assert!(accounts[0].active);
}

#[test]
fn test_parse_gh_accounts_other_host() {
    let accounts = parse_gh_accounts(GITHUB_STATUS_OTHER_HOST);
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].name, "mlucianoCAE");
    assert!(accounts[0].active);
}

#[test]
fn test_parse_gh_accounts_empty() {
    let accounts = parse_gh_accounts("");
    assert!(accounts.is_empty());
}

#[test]
fn test_next_account_none_if_less_than_two() {
    let accounts = vec![GhAccount {
        name: "a".into(),
        active: true,
    }];
    assert_eq!(next_account(&accounts), None);
}

#[test]
fn test_next_account_none_if_no_active() {
    let accounts = vec![
        GhAccount {
            name: "a".into(),
            active: false,
        },
        GhAccount {
            name: "b".into(),
            active: false,
        },
    ];
    assert_eq!(next_account(&accounts), None);
}

#[test]
fn test_next_account_wrap_two() {
    let accounts = vec![
        GhAccount {
            name: "a".into(),
            active: true,
        },
        GhAccount {
            name: "b".into(),
            active: false,
        },
    ];
    assert_eq!(next_account(&accounts), Some("b"));
}

#[test]
fn test_next_account_wrap_three() {
    let accounts = vec![
        GhAccount {
            name: "a".into(),
            active: false,
        },
        GhAccount {
            name: "b".into(),
            active: true,
        },
        GhAccount {
            name: "c".into(),
            active: false,
        },
    ];
    assert_eq!(next_account(&accounts), Some("c"));
}

#[test]
fn test_next_account_wrap_three_last_active() {
    let accounts = vec![
        GhAccount {
            name: "a".into(),
            active: false,
        },
        GhAccount {
            name: "b".into(),
            active: false,
        },
        GhAccount {
            name: "c".into(),
            active: true,
        },
    ];
    assert_eq!(next_account(&accounts), Some("a"));
}

#[test]
fn test_chip_view_no_accounts() {
    let view = chip_view(&[], None);
    assert_eq!(view.label, "gh: none");
    assert_eq!(view.tone, ChipTone::Missing);
}

#[test]
fn test_chip_view_active_no_owner() {
    let accounts = vec![GhAccount {
        name: "mlucianoCAE".into(),
        active: true,
    }];
    let view = chip_view(&accounts, None);
    assert_eq!(view.label, "mlucianoCAE");
    assert_eq!(view.tone, ChipTone::Unknown);
}

#[test]
fn test_chip_view_match_case_insensitive() {
    let accounts = vec![GhAccount {
        name: "mlucianoCAE".into(),
        active: true,
    }];
    let view = chip_view(&accounts, Some("MLucianoCAE"));
    assert_eq!(view.label, "mlucianoCAE");
    assert_eq!(view.tone, ChipTone::Match);
}

#[test]
fn test_chip_view_mismatch() {
    let accounts = vec![GhAccount {
        name: "mlucianoCAE".into(),
        active: true,
    }];
    let view = chip_view(&accounts, Some("other"));
    assert_eq!(view.label, "mlucianoCAE");
    assert_eq!(view.tone, ChipTone::Mismatch);
}

#[test]
fn test_chip_view_match_exact() {
    let accounts = vec![GhAccount {
        name: "mlucianoCAE".into(),
        active: true,
    }];
    let view = chip_view(&accounts, Some("mlucianoCAE"));
    assert_eq!(view.label, "mlucianoCAE");
    assert_eq!(view.tone, ChipTone::Match);
}

#[test]
fn test_chip_view_multiple_accounts_active_second() {
    let accounts = vec![
        GhAccount {
            name: "first".into(),
            active: false,
        },
        GhAccount {
            name: "second".into(),
            active: true,
        },
    ];
    let view = chip_view(&accounts, Some("second"));
    assert_eq!(view.label, "second");
    assert_eq!(view.tone, ChipTone::Match);
}

#[test]
fn test_chip_view_multiple_accounts_active_second_mismatch() {
    let accounts = vec![
        GhAccount {
            name: "first".into(),
            active: false,
        },
        GhAccount {
            name: "second".into(),
            active: true,
        },
    ];
    let view = chip_view(&accounts, Some("first"));
    assert_eq!(view.label, "second");
    assert_eq!(view.tone, ChipTone::Mismatch);
}
