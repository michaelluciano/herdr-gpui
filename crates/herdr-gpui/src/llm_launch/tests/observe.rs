use super::*;
use crate::config::llm_launchers::{Kind, Launcher};
use Health::{Down, Loading, Ready};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    time::{Duration, Instant},
};

const REAL_LINE: &str = "  67296 llama-server -hf unsloth/gpt-oss-20b-GGUF:F16 --host 127.0.0.1 --port 8081 -a coder -c 65536 -ngl 999 -fa on -ctk q8_0 -ctv q8_0 --jinja --temp 1.0 --top-p 1.0 --top-k 0 --min-p 0 --reasoning-effort medium --reasoning-format deepseek";

fn llama(host: &str, group: Option<&str>, port: u16) -> Launcher {
    Launcher {
        label: format!("m{port}"),
        host: host.to_owned(),
        group: group.map(str::to_owned),
        kind: Kind::Llama {
            port,
            start: "start".to_owned(),
            url: None,
        },
    }
}

fn terminal(host: &str) -> Launcher {
    Launcher {
        label: "t".to_owned(),
        host: host.to_owned(),
        group: None,
        kind: Kind::Terminal {
            run: "ls".to_owned(),
        },
    }
}

// parse_llama_ports

#[test]
fn the_real_llama_server_line_gives_its_port() {
    assert_eq!(parse_llama_ports(REAL_LINE), [8081]);
}

#[test]
fn a_full_ps_listing_keeps_only_llama_servers() {
    let ps = format!(
        "    1 /sbin/launchd\n{REAL_LINE}\n  400 /opt/homebrew/bin/llama-server --port=8082 -m x.gguf\n  500 grep llama-server --port 9000\n"
    );
    assert_eq!(parse_llama_ports(&ps), [8081, 8082]);
}

#[test]
fn only_an_exact_llama_server_basename_counts() {
    let ps = "\
 10 llama-server-wrapper --port 8081
 11 grep llama-server --port 8082
 12 vim llama-server.log --port 8083
 13 /usr/local/bin/not-llama-server --port 8084
 14 /usr/local/bin/llama-server --port 8085
";
    assert_eq!(parse_llama_ports(ps), [8085]);
}

#[test]
fn both_port_spellings_are_read() {
    let ps = " 1 llama-server --port 8081\n 2 llama-server --port=8082\n";
    assert_eq!(parse_llama_ports(ps), [8081, 8082]);
}

#[test]
fn lines_without_a_valid_port_are_skipped() {
    let ps = "\
 1 llama-server -m x.gguf
 2 llama-server --port
 3 llama-server --port abc
 4 llama-server --port 0
 5 llama-server --port 70000
 6 llama-server --port=
 7 llama-server --port=-1
 8 llama-server --port 65535
 9 llama-server --port 1
";
    assert_eq!(parse_llama_ports(ps), [65535, 1]);
}

#[test]
fn garbage_and_empty_input_give_nothing_without_panicking() {
    assert!(parse_llama_ports("").is_empty());
    assert!(parse_llama_ports("\n\n   \n").is_empty());
    assert!(parse_llama_ports("llama-server").is_empty());
    assert!(parse_llama_ports("\u{0}\u{1}\u{ff}\u{feff} --port 8081 ☃☃").is_empty());
    assert!(parse_llama_ports("12").is_empty());
}

#[test]
fn ports_are_deduplicated_in_first_seen_order() {
    let ps =
        " 1 llama-server --port 8083\n 2 llama-server --port 8081\n 3 llama-server --port 8083\n";
    assert_eq!(parse_llama_ports(ps), [8083, 8081]);
}

#[test]
fn at_most_32_ports_are_returned() {
    let ps: String = (0..40)
        .map(|i| format!(" {i} llama-server --port {}\n", 9000 + i))
        .collect();
    let ports = parse_llama_ports(&ps);
    assert_eq!(ports.len(), 32);
    assert_eq!(ports[0], 9000);
    assert_eq!(ports[31], 9031);
}

#[test]
fn only_the_first_mebibyte_is_considered() {
    let filler = format!(" 1 sleep {}\n", "x".repeat(1000));
    let mut ps = filler.repeat((1 << 20) / filler.len() + 1);
    ps.push_str(" 2 llama-server --port 8081\n");
    assert!(parse_llama_ports(&ps).is_empty());
    let early = format!(" 2 llama-server --port 8082\n{ps}");
    assert_eq!(parse_llama_ports(&early), [8082]);
}

// foreign_ports

#[test]
fn foreign_ports_are_the_running_ones_no_row_names() {
    assert_eq!(foreign_ports(&[8081, 8082, 8083], &[8082]), [8081, 8083]);
    assert!(foreign_ports(&[8081], &[8081]).is_empty());
    assert!(foreign_ports(&[], &[8081]).is_empty());
    assert_eq!(foreign_ports(&[8083, 8081], &[]), [8083, 8081]);
}

// observe_with

fn ps_with(ports: &[u16]) -> String {
    ports
        .iter()
        .map(|p| format!(" 1 llama-server --port {p}\n"))
        .collect()
}

#[test]
fn only_local_llama_rows_are_probed() {
    let rows = [
        llama("Local", None, 8081),
        terminal("Local"),
        llama("remoteluci", None, 8082),
        llama("local", None, 8083),
    ];
    let probed = std::cell::RefCell::new(Vec::new());
    let observed = observe_with(&rows, "", |port| {
        probed.borrow_mut().push(port);
        Ready
    });
    assert_eq!(*probed.borrow(), [8081, 8083]);
    assert_eq!(observed.health, [Some(Ready), None, None, Some(Ready)]);
}

#[test]
fn each_row_gets_its_own_probe_result() {
    let rows = [llama("Local", None, 8081), llama("Local", None, 8082)];
    let observed = observe_with(&rows, "", |port| if port == 8081 { Loading } else { Down });
    assert_eq!(observed.health, [Some(Loading), Some(Down)]);
}

#[test]
fn foreign_is_what_runs_locally_and_no_local_row_names() {
    let rows = [llama("Local", None, 8081), llama("remoteluci", None, 8083)];
    let observed = observe_with(&rows, &ps_with(&[8081, 8082, 8083]), |_| Down);
    // 8083 is named only by a row on another machine, so it is foreign here.
    assert_eq!(observed.foreign, [8082, 8083]);
}

#[test]
fn nothing_to_watch_means_no_ps_and_no_probes() {
    let rows = [terminal("Local"), llama("remoteluci", None, 8081)];
    assert!(!needs_ps(&rows));
    assert!(!needs_ps(&[]));
    assert!(needs_ps(&[terminal("Local"), llama("Local", None, 8081)]));
    assert!(needs_ps(&[llama("local", None, 8081)]));
    let observed = observe_with(&rows, &ps_with(&[8085]), |_| panic!("probed"));
    assert_eq!(observed.health, [None, None]);
}

// states

fn observed(health: Vec<Option<Health>>, foreign: Vec<u16>) -> Observed {
    Observed { health, foreign }
}

#[test]
fn states_follow_the_observed_health() {
    let rows = [
        llama("Local", Some("g"), 8081),
        llama("Local", Some("g"), 8082),
        terminal("Local"),
    ];
    let seen = observed(vec![Some(Ready), Some(Down), None], vec![]);
    assert_eq!(
        states(&rows, &seen),
        [
            RowState::Running,
            RowState::Blocked {
                by: Blocker::Row(0)
            },
            RowState::Stopped
        ]
    );
}

#[test]
fn a_foreign_local_server_blocks_a_local_group_row() {
    let rows = [llama("Local", Some("g"), 8081)];
    let seen = observed(vec![Some(Down)], vec![8085]);
    assert_eq!(
        states(&rows, &seen),
        [RowState::Blocked {
            by: Blocker::Foreign { port: 8085 }
        }]
    );
}

#[test]
fn an_observation_of_other_row_counts_does_not_panic() {
    let four = observed(vec![Some(Ready), Some(Down), None, Some(Down)], vec![]);
    let two = [
        llama("Local", Some("g"), 8081),
        llama("Local", Some("g"), 8082),
    ];
    let six: Vec<Launcher> = (0..6).map(|i| llama("Local", None, 8081 + i)).collect();
    // Extra observed entries are ignored.
    assert_eq!(
        states(&two, &four),
        [
            RowState::Running,
            RowState::Blocked {
                by: Blocker::Row(0)
            }
        ]
    );
    // Missing ones read as not probed.
    let padded = states(&six, &four);
    assert_eq!(padded.len(), 6);
    assert_eq!(padded[0], RowState::Running);
    assert_eq!(&padded[4..], [RowState::Stopped, RowState::Stopped]);
}

// probe_health

const SHORT: Duration = Duration::from_millis(300);

/// A localhost server that answers one request with `reply`.
fn answering(reply: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    port
}

fn status_reply(status: &'static str) -> &'static str {
    match status {
        "200" => {
            "HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"status\":\"ok\"}"
        }
        "503" => {
            "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}"
        }
        _ => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    }
}

#[test]
fn a_200_answer_is_ready() {
    let port = answering(status_reply("200"));
    assert_eq!(probe_health(port, SHORT), Ready);
}

#[test]
fn a_503_answer_is_loading() {
    let port = answering(status_reply("503"));
    assert_eq!(probe_health(port, SHORT), Loading);
}

#[test]
fn a_404_answer_is_down() {
    let port = answering(status_reply("404"));
    assert_eq!(probe_health(port, SHORT), Down);
}

#[test]
fn a_listener_that_never_answers_is_down_within_the_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    listener.set_nonblocking(true).expect("nonblocking");
    let (release, released) = mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        // Accepts once and then says nothing; gives up after 3 s so a probe
        // that never connects cannot hang the test.
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut held = None;
        while held.is_none() && Instant::now() < deadline {
            match listener.accept() {
                Ok(pair) => held = Some(pair),
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
        let _ = released.recv_timeout(Duration::from_secs(3));
        drop(held);
    });
    let started = Instant::now();
    let health = probe_health(port, SHORT);
    let took = started.elapsed();
    let _ = release.send(());
    holder.join().expect("holder");
    assert_eq!(health, Down);
    assert!(took < Duration::from_secs(2), "took {took:?}");
}

#[test]
fn a_closed_port_is_down() {
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.local_addr().expect("addr").port()
    };
    assert_eq!(probe_health(port, SHORT), Down);
}

// run_ps

#[test]
fn ps_lists_this_test_process() {
    let output = run_ps();
    let pid = std::process::id().to_string();
    assert!(
        output
            .lines()
            .any(|line| line.split_whitespace().next() == Some(pid.as_str())),
        "no line for pid {pid} in {} bytes",
        output.len()
    );
    assert!(output.len() <= 1 << 20);
}
