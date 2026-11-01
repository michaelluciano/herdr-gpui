use super::*;

#[test]
fn open_graph_is_a_gui_command_with_no_default_shortcut() {
    let info = COMMANDS
        .iter()
        .find(|info| info.command == Command::OpenGraph)
        .expect("the catalog lists OpenGraph");
    assert_eq!(info.name, "open_graph");
    assert!(!info.label.is_empty());
    assert!(info.base_shortcuts().is_empty());
    assert_eq!(request(Command::OpenGraph, &snapshot()), None);
    assert_eq!(
        serde_json::from_value::<Command>(json!("OpenGraph")).unwrap(),
        Command::OpenGraph
    );
}
