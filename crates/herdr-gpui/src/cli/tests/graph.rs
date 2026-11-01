use super::*;

#[test]
fn graph_is_a_control_command_with_shared_flags() {
    assert_eq!(
        LaunchOptions::parse(["graph", "open", "--json", "--window", "7"])
            .unwrap()
            .mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Graph,
            json: true,
            window: Some(7),
        })
    );
    // `open` may be left out, as for `review`.
    assert_eq!(
        LaunchOptions::parse(["graph"]).unwrap().mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Graph,
            json: false,
            window: None,
        })
    );
    assert_eq!(
        LaunchOptions::parse(["graph", "--help"]).unwrap().mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Help(Some("graph")),
            json: false,
            window: None,
        })
    );
    for args in [["graph", "bad"], ["graph", "--unknown"]] {
        assert!(LaunchOptions::parse(args).is_err());
    }
}
