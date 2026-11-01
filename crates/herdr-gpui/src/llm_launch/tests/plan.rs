use super::*;
use crate::config::llm_launchers::{Kind, Launcher};

fn fixture() -> Vec<Launcher> {
    let llama = |label: &str, host: &str, port| Launcher {
        label: label.into(),
        host: host.into(),
        group: Some("mac-llm".into()),
        kind: Kind::Llama {
            port,
            start: "start".into(),
            url: None,
        },
    };
    vec![
        llama("coder", "Local", 8081),
        llama("reason", "Local", 8082),
        llama("gemma", "Local", 8083),
        Launcher {
            label: "nemotron".into(),
            host: "remoteluci".into(),
            group: None,
            kind: Kind::Terminal {
                run: "ollama run nemotron-3-nano:4b".into(),
            },
        },
    ]
}

#[test]
fn each_state_has_one_click_plan() {
    for (state, expected) in [
        (RowState::Running, Plan::Open),
        (RowState::Stopped, Plan::StartThenOpen),
        (
            RowState::Blocked {
                by: Blocker::Row(2),
            },
            Plan::Refuse {
                by: Blocker::Row(2),
            },
        ),
        (
            RowState::Blocked {
                by: Blocker::Foreign { port: 9 },
            },
            Plan::Refuse {
                by: Blocker::Foreign { port: 9 },
            },
        ),
        (RowState::Starting, Plan::Ignore),
    ] {
        assert_eq!(plan(state), expected, "{state:?}");
    }
}

#[test]
fn the_default_address_is_the_loopback_port() -> crate::Result<()> {
    assert_eq!(
        url_for(8081, None)?,
        crate::browser::WebUrl::from_typed("http://127.0.0.1:8081/")?
    );
    Ok(())
}

#[test]
fn an_explicit_address_is_used_instead_of_the_port() -> crate::Result<()> {
    let url = url_for(8081, Some("http://localhost:9000/ui"))?;
    assert_eq!(
        url,
        crate::browser::WebUrl::from_typed("http://localhost:9000/ui")?
    );
    assert_ne!(url, url_for(8081, None)?);
    Ok(())
}

#[test]
fn an_invalid_address_is_refused() {
    for bad in ["ftp://example.com/", "http://", "not a url://x"] {
        assert!(
            matches!(
                url_for(8081, Some(bad)),
                Err(crate::Error::InvalidBrowserUrl)
            ),
            "{bad}"
        );
    }
}

#[test]
fn rows_for_host_keeps_order_and_original_indices() {
    let launchers = fixture();
    let rows = rows_for_host(&launchers, "Local", "local");
    let indices: Vec<usize> = rows.iter().map(|(index, _)| *index).collect();
    assert_eq!(indices, [0, 1, 2]);
    assert_eq!(rows[2].1.label, "gemma");

    let rows = rows_for_host(&launchers, "remoteluci", "ssh:remoteluci");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, 3);
    assert_eq!(rows[0].1.label, "nemotron");

    assert!(rows_for_host(&launchers, "elsewhere", "ssh:elsewhere").is_empty());
}
