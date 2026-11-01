use super::*;
use crate::config::llm_launchers::{Error, Kind, Launcher, MAX_ROWS};

const FIXTURE: &str = r#"
[[llm_launchers]]
label = "coder · gpt-oss-20b"
host = "Local"
kind = "llama"
group = "mac-llm"
port = 8081
start = "$HOME/llm/llm coder"
[[llm_launchers]]
label = "reason · Qwen3.8-27B"
host = "Local"
kind = "llama"
group = "mac-llm"
port = 8082
start = "$HOME/llm/llm reason"
[[llm_launchers]]
label = "gemma · Gemma 4 26B-A4B"
host = "Local"
kind = "llama"
group = "mac-llm"
port = 8083
start = "LLM_CONF=$HOME/llm/models.gemma.conf $HOME/llm/llm reason"
[[llm_launchers]]
label = "nemotron-3-nano:4b"
host = "remoteluci"
kind = "terminal"
run = "ollama run nemotron-3-nano:4b"
"#;

fn row(extra: &str) -> String {
    format!("[[llm_launchers]]\n{extra}\n")
}

fn llama(label: &str, start: &str) -> String {
    row(&format!(
        "label = {label:?}\nhost = \"Local\"\nkind = \"llama\"\nport = 8081\nstart = {start:?}"
    ))
}

fn terminal(run: &str) -> String {
    row(&format!(
        "label = \"t\"\nhost = \"Local\"\nkind = \"terminal\"\nrun = {run:?}"
    ))
}

#[test]
fn the_operators_four_rows_parse_in_order() -> anyhow::Result<()> {
    let config = Config::parse(FIXTURE)?;
    assert_eq!(config.llm_launchers.len(), 4);
    assert_eq!(
        config.llm_launchers[0],
        Launcher {
            label: "coder · gpt-oss-20b".into(),
            host: "Local".into(),
            group: Some("mac-llm".into()),
            kind: Kind::Llama {
                port: 8081,
                start: "$HOME/llm/llm coder".into(),
                url: None,
            },
        }
    );
    assert_eq!(config.llm_launchers[1].label, "reason · Qwen3.8-27B");
    assert_eq!(config.llm_launchers[2].label, "gemma · Gemma 4 26B-A4B");
    assert_eq!(
        config.llm_launchers[3],
        Launcher {
            label: "nemotron-3-nano:4b".into(),
            host: "remoteluci".into(),
            group: None,
            kind: Kind::Terminal {
                run: "ollama run nemotron-3-nano:4b".into()
            },
        }
    );
    Ok(())
}

#[test]
fn an_explicit_url_is_kept() -> anyhow::Result<()> {
    let config = Config::parse(&row(
        "label = \"a\"\nhost = \"Local\"\nkind = \"llama\"\nport = 9\nstart = \"s\"\nurl = \"http://x:1/\"",
    ))?;
    assert!(matches!(
        &config.llm_launchers[0].kind,
        Kind::Llama { url: Some(url), .. } if url == "http://x:1/"
    ));
    Ok(())
}

#[test]
fn an_absent_or_empty_table_gives_no_launchers() -> anyhow::Result<()> {
    assert!(Config::default().llm_launchers.is_empty());
    assert!(Config::parse("")?.llm_launchers.is_empty());
    assert!(Config::parse(DEFAULT_CONFIG)?.llm_launchers.is_empty());
    assert!(
        Config::parse("llm_launchers = []")?
            .llm_launchers
            .is_empty()
    );
    Ok(())
}

#[test]
fn an_unknown_key_in_a_row_is_reported_not_fatal() -> anyhow::Result<()> {
    let config = Config::parse(&terminal("ls").replace("kind", "bogus = 1\nkind"))?;
    assert_eq!(config.llm_launchers.len(), 1);
    assert!(
        config
            .unknown_keys
            .contains(&"llm_launchers.0.bogus".to_owned()),
        "{:?}",
        config.unknown_keys
    );
    Ok(())
}

fn error(text: &str) -> Option<Error> {
    match Config::parse(text) {
        Err(crate::Error::LlmLaunchers(error)) => Some(error),
        _ => None,
    }
}

#[test]
fn each_malformed_row_is_refused_with_its_own_error() {
    let long_label = "x".repeat(81);
    let long_command = "x".repeat(513);
    assert!(matches!(
        error(&llama("", "s")),
        Some(Error::Empty {
            row: 0,
            field: "label"
        })
    ));
    assert!(matches!(
        error(&llama(&long_label, "s")),
        Some(Error::TooLong {
            row: 0,
            field: "label",
            max: 80
        })
    ));
    assert!(matches!(
        error(&llama("a\tb", "s")),
        Some(Error::ControlChar {
            row: 0,
            field: "label"
        })
    ));
    assert!(matches!(
        error(&llama("a", "")),
        Some(Error::Empty {
            row: 0,
            field: "start"
        })
    ));
    assert!(matches!(
        error(&llama("a", &long_command)),
        Some(Error::TooLong {
            row: 0,
            field: "start",
            max: 512
        })
    ));
    assert!(matches!(
        error(&terminal("ls\nrm")),
        Some(Error::ControlChar {
            row: 0,
            field: "run"
        })
    ));
    assert!(matches!(
        error(&terminal("")),
        Some(Error::Empty {
            row: 0,
            field: "run"
        })
    ));
}

#[test]
fn ports_outside_one_to_65535_are_refused() {
    for port in ["0", "70000", "-1"] {
        let text = row(&format!(
            "label = \"a\"\nhost = \"Local\"\nkind = \"llama\"\nport = {port}\nstart = \"s\""
        ));
        assert!(
            matches!(error(&text), Some(Error::InvalidPort { row: 0 })),
            "{port}"
        );
    }
}

#[test]
fn kind_and_host_are_required_and_kind_is_closed() {
    assert!(matches!(
        error(&row("label = \"a\"\nhost = \"Local\"\nrun = \"ls\"")),
        Some(Error::MissingField {
            row: 0,
            field: "kind"
        })
    ));
    assert!(matches!(
        error(&row("label = \"a\"\nhost = \"Local\"\nkind = \"other\"")),
        Some(Error::UnknownKind { row: 0, ref kind }) if kind == "other"
    ));
    assert!(matches!(
        error(&row("label = \"a\"\nkind = \"terminal\"\nrun = \"ls\"")),
        Some(Error::MissingField {
            row: 0,
            field: "host"
        })
    ));
}

#[test]
fn a_row_carries_only_its_kinds_fields() {
    let llama_with_run = row(
        "label = \"a\"\nhost = \"Local\"\nkind = \"llama\"\nport = 1\nstart = \"s\"\nrun = \"ls\"",
    );
    assert!(matches!(
        error(&llama_with_run),
        Some(Error::WrongField {
            row: 0,
            field: "run",
            ..
        })
    ));
    let terminal_with_port =
        row("label = \"a\"\nhost = \"Local\"\nkind = \"terminal\"\nrun = \"ls\"\nport = 8081");
    assert!(matches!(
        error(&terminal_with_port),
        Some(Error::WrongField {
            row: 0,
            field: "port",
            ..
        })
    ));
}

#[test]
fn rows_past_the_limit_are_dropped_and_named() -> anyhow::Result<()> {
    let text = terminal("ls").repeat(MAX_ROWS + 1);
    let config = Config::parse(&text)?;
    assert_eq!(config.llm_launchers.len(), MAX_ROWS);
    let named: Vec<&String> = config
        .unknown_keys
        .iter()
        .filter(|key| key.starts_with("llm_launchers."))
        .collect();
    assert_eq!(named, [&format!("llm_launchers.{MAX_ROWS}")]);
    Ok(())
}

#[test]
fn a_later_layer_replaces_the_array() -> anyhow::Result<()> {
    let config = Config::parse_layers([FIXTURE, terminal("top").as_str()], &Daemon::default())?;
    assert_eq!(config.llm_launchers.len(), 1);
    assert!(matches!(
        &config.llm_launchers[0].kind,
        Kind::Terminal { run } if run == "top"
    ));
    let config = Config::parse_layers([FIXTURE, "llm_launchers = []"], &Daemon::default())?;
    assert!(config.llm_launchers.is_empty());
    Ok(())
}

#[test]
fn a_host_matches_by_label_or_id_exactly() {
    let launcher = |host: &str| Launcher {
        label: "a".into(),
        host: host.into(),
        group: None,
        kind: Kind::Terminal { run: "ls".into() },
    };
    assert!(launcher("Local").on_host("Local", "local"));
    assert!(launcher("local").on_host("Local", "local"));
    assert!(launcher("local").on_host("This Mac", "local"));
    assert!(launcher("Local").on_host("This Mac", "local"));
    assert!(launcher("remoteluci").on_host("remoteluci", "ssh:remoteluci"));
    assert!(launcher("ssh:remoteluci").on_host("remoteluci", "ssh:remoteluci"));
    assert!(!launcher("remoteluci").on_host("Local", "local"));
    assert!(!launcher("Remoteluci").on_host("remoteluci", "ssh:remoteluci"));
}
