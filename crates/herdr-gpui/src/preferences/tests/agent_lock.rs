//! The agent order lock is remembered with the rest of the chrome.
use super::*;

#[core::prelude::v1::test]
fn the_lock_and_its_order_round_trip_and_damage_loses_only_itself() {
    let directory = TestDirectory::new();
    let path = directory.0.join("preferences.json");
    let chrome = Chrome {
        sidebar_width: Some(250.),
        agent_lock: true,
        agent_frozen: vec!["p2".into(), "p1".into()],
        ..Chrome::default()
    };
    write_chrome(&path, chrome.clone()).unwrap();
    assert_eq!(read_chrome(&path).unwrap(), chrome);
    // Files from earlier builds have neither.
    fs::write(&path, r#"{"sidebar_width_px": 250}"#).unwrap();
    let read = read_chrome(&path).unwrap();
    assert!(!read.agent_lock);
    assert!(read.agent_frozen.is_empty());
    for damaged in [r#""p1""#, "7", "null", r#"{"p1": 1}"#] {
        fs::write(
            &path,
            format!(
                r#"{{"sidebar_width_px": 250, "agent_lock": {damaged}, "agent_frozen": {damaged}}}"#
            ),
        )
        .unwrap();
        let read = read_chrome(&path).unwrap();
        assert!(!read.agent_lock, "{damaged}");
        assert!(read.agent_frozen.is_empty(), "{damaged}");
        assert_eq!(read.sidebar_width, Some(250.), "{damaged}");
    }
    // Entries that are not IDs are dropped; the rest keep their order.
    let long = "x".repeat(MAX_DEVICE_ID + 1);
    fs::write(
        &path,
        format!(r#"{{"agent_frozen": ["p2", 3, "", "{long}", "p1"]}}"#),
    )
    .unwrap();
    assert_eq!(read_chrome(&path).unwrap().agent_frozen, ["p2", "p1"]);
}
