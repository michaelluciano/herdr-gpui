use super::*;

#[test]
fn a_row_opens_its_file_without_a_line() {
    let handoff = Handoff {
        path: std::env::temp_dir().join("p/sessions/a.md"),
        project: "p".into(),
        id: "a".into(),
        created: "2026-10-31".into(),
    };
    let target = target_for(&handoff);
    assert_eq!(target.path, handoff.path);
    assert_eq!(target.line, None);
}
