use super::*;

#[test]
fn a_row_opens_its_file_without_a_line() {
    let handoff = Handoff {
        path: std::env::temp_dir().join("p/sessions/a.md"),
        ..fixture("a", "p", "2026-10-31", "open")
    };
    let target = target_for(&handoff);
    assert_eq!(target.path, handoff.path);
    assert_eq!(target.line, None);
}
