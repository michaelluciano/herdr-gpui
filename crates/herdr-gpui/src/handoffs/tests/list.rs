use super::*;
use crate::handoffs::list::MAX_HANDOFFS;

fn ids(list: &[Handoff]) -> Vec<&str> {
    list.iter().map(|handoff| handoff.id.as_str()).collect()
}

#[test]
fn a_handoff_is_listed_with_its_front_matter() {
    let root = Root::new("one");
    let path = root.session(
        "claude-setup",
        "2026-10-31-e8da4cd3.md",
        CLAUDE_SETUP_HEADER,
    );
    let list = list_all(root.path());
    assert_eq!(
        list,
        vec![Handoff {
            path,
            project: "claude-setup".into(),
            id: "S-2026-10-31-e8da4cd3".into(),
            created: "2026-10-31T13:41:42".into(),
            status: "open".into(),
            session_id: "e8da4cd3-a82f-45cf-83fc-da30f6606ae9".into(),
            parent: "P-claude-setup".into(),
            prev: String::new(),
            claimed_by: String::new(),
            kind: Kind::Legacy,
            summary: "Body text.".into(),
        }]
    );
}

#[test]
fn a_claimed_handoff_is_listed_with_its_claim_and_prev() {
    let root = Root::new("claimed");
    let path = root.session("kb", "S37-lease-rerun-2026-10-31.md", KB_HEADER_CLAIMED);
    let list = list_all(root.path());
    assert_eq!(list.len(), 1);
    let handoff = &list[0];
    assert_eq!(handoff.path, path);
    assert_eq!(handoff.status, "claimed");
    assert_eq!(handoff.parent, "kb");
    assert_eq!(handoff.prev, "S36-lease-w4-t157-2026-10-31");
    assert_eq!(
        handoff.session_id,
        "pending (S37 claims with bin/bh build claim S37)"
    );
    assert_eq!(handoff.claimed_by, "184567ac-2533-4202-94f2-bf3b35b0b63e");
}

#[test]
fn every_status_is_loaded_and_kept_verbatim() {
    let root = Root::new("status");
    let with = |id: &str, status: &str| format!("---\nid: {id}\nstatus: {status}\n---\n");
    root.session("p", "padded.md", with("padded", "  open  "));
    root.session("p", "upper.md", with("upper", "OPEN"));
    root.session("p", "accepted.md", with("accepted", "accepted"));
    root.session("p", "empty.md", with("empty", ""));
    let mut found: Vec<(String, String)> = list_all(root.path())
        .into_iter()
        .map(|handoff| (handoff.id, handoff.status))
        .collect();
    found.sort_unstable();
    assert_eq!(
        found,
        [
            ("accepted".to_owned(), "accepted".to_owned()),
            ("empty".to_owned(), String::new()),
            ("padded".to_owned(), "open".to_owned()),
            ("upper".to_owned(), "OPEN".to_owned()),
        ]
    );
}

#[test]
fn files_without_front_matter_or_status_are_not_listed() {
    let root = Root::new("shape");
    root.session("p", "plain.md", "# no front matter\nstatus: open\n");
    root.session(
        "p",
        "nostatus.md",
        "---\nid: nostatus\ncreated: 2026-10-31\n---\n",
    );
    root.session(
        "p",
        "unclosed.md",
        "---\nid: unclosed\nstatus: open\nno closing fence\n",
    );
    root.session("p", "late.md", "\n---\nid: late\nstatus: open\n---\n");
    root.session("p", "empty.md", "");
    assert!(list_all(root.path()).is_empty());
}

#[test]
fn the_id_falls_back_to_the_file_stem() {
    let root = Root::new("stem");
    root.session(
        "p",
        "no-id.md",
        "---\nstatus: open\ncreated: 2026-10-30\n---\n",
    );
    root.session("p", "blank-id.md", "---\nid:   \nstatus: open\n---\n");
    let list = list_all(root.path());
    let mut found = ids(&list);
    found.sort_unstable();
    assert_eq!(found, ["blank-id", "no-id"]);
    let plain = list.iter().find(|handoff| handoff.id == "no-id").unwrap();
    assert_eq!(plain.created, "2026-10-30");
    let blank = list
        .iter()
        .find(|handoff| handoff.id == "blank-id")
        .unwrap();
    assert_eq!(blank.created, "", "a missing created is empty");
}

#[test]
fn only_session_markdown_files_one_level_down_count() {
    let root = Root::new("depth");
    let open = open_header("keep", "2026-10-31");
    root.session("proj", "keep.md", &open);
    root.session("proj", "keep.md.claim", &open);
    root.session("proj", "keep.txt", &open);
    root.session("proj", "keep.MD.bak", &open);
    root.session("proj/nested", "deep.md", &open);
    root.file("INDEX.md", &open);
    root.file("proj/INDEX.md", &open);
    root.file("proj/HANDOFF-2026.md", &open);
    root.file("proj/notes/x.md", &open);
    root.file("proj/other/sessions/y.md", &open);
    root.file("sessions/z.md", &open);
    let list = list_all(root.path());
    assert_eq!(ids(&list), ["keep"]);
    assert_eq!(list[0].project, "proj");
}

#[cfg(unix)]
#[test]
fn symlinks_are_not_followed() {
    let root = Root::new("links");
    let real = root.file("elsewhere/real.md", open_header("real", "2026-10-31"));
    root.session("p", "ok.md", open_header("ok", "2026-10-31"));
    std::fs::create_dir_all(root.path().join("p/sessions")).unwrap();
    std::os::unix::fs::symlink(&real, root.path().join("p/sessions/link.md")).unwrap();
    std::os::unix::fs::symlink(root.path().join("elsewhere"), root.path().join("q")).unwrap();
    assert_eq!(ids(&list_all(root.path())), ["ok"]);
}

#[test]
fn a_directory_named_like_a_file_is_skipped() {
    let root = Root::new("dirmd");
    std::fs::create_dir_all(root.path().join("p/sessions/dir.md")).unwrap();
    root.session("p", "ok.md", open_header("ok", "2026-10-31"));
    assert_eq!(ids(&list_all(root.path())), ["ok"]);
}

#[test]
fn a_missing_or_empty_root_lists_nothing() {
    let root = Root::new("missing");
    assert!(list_all(&root.path().join("absent")).is_empty());
    assert!(list_all(root.path()).is_empty());
    let file = root.file("a-file", "x");
    assert!(list_all(&file).is_empty());
}

#[test]
fn newest_first_by_created_with_path_as_the_tie_break() {
    let root = Root::new("order");
    root.session("a", "old.md", open_header("old", "2026-10-01"));
    root.session("a", "mid.md", open_header("mid", "2026-10-31"));
    root.session("b", "new.md", open_header("new", "2026-10-31T13:41:42"));
    root.session("b", "tie-b.md", open_header("tie-b", "2026-10-15"));
    root.session("a", "tie-a.md", open_header("tie-a", "2026-10-15"));
    root.session("a", "blank.md", "---\nid: blank\nstatus: open\n---\n");
    assert_eq!(
        ids(&list_all(root.path())),
        ["new", "mid", "tie-a", "tie-b", "old", "blank"]
    );
}

#[test]
fn at_most_two_hundred_are_kept_and_they_are_the_newest() {
    let root = Root::new("cap");
    for n in 0..230 {
        let name = format!("s{n:03}.md");
        root.session(
            "p",
            &name,
            open_header(
                &format!("s{n:03}"),
                &format!("2026-09-01T{:02}:{:02}:00", n / 60, n % 60),
            ),
        );
    }
    let list = list_all(root.path());
    assert_eq!(list.len(), MAX_HANDOFFS);
    assert_eq!(list.len(), 200);
    let created: Vec<_> = list
        .iter()
        .map(|handoff| handoff.created.as_str())
        .collect();
    let mut sorted = created.clone();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(created, sorted, "newest first");
    // The newest of all 230 is kept, the thirty oldest are dropped.
    assert_eq!(created.first(), Some(&"2026-09-01T03:49:00"));
    assert_eq!(created.last(), Some(&"2026-09-01T00:30:00"));
}

#[test]
fn only_the_first_four_kib_of_a_file_is_read() {
    let root = Root::new("bound");
    let mut late = String::from("---\nid: late\n");
    late.push_str(&"filler: x\n".repeat(600));
    late.push_str("status: open\n---\n");
    assert!(late.len() > 4096);
    root.session("p", "late.md", &late);
    // A big body after a short header is fine.
    let mut big = open_header("big", "2026-10-31");
    big.push_str(&"body line\n".repeat(50_000));
    root.session("p", "big.md", &big);
    assert_eq!(ids(&list_all(root.path())), ["big"]);
}

#[test]
fn unreadable_or_non_utf8_files_are_skipped_without_a_panic() {
    let root = Root::new("bytes");
    root.session("p", "bad.md", b"---\nid: \xff\xfe\nstatus: open\n---\n");
    root.session("p", "binary.md", [0u8, 159, 146, 150, 0, 1, 2]);
    root.session("p", "ok.md", open_header("ok", "2026-10-31"));
    assert_eq!(ids(&list_all(root.path())), ["ok"]);
}

#[test]
fn a_four_kib_cut_inside_a_multibyte_character_still_parses() {
    let root = Root::new("cut");
    let mut text = open_header("cut", "2026-10-31");
    let pad = 4095 - text.len();
    text.push_str(&"a".repeat(pad));
    text.push('é');
    text.push_str("tail\n");
    assert!(text.len() > 4096);
    root.session("p", "cut.md", &text);
    assert_eq!(ids(&list_all(root.path())), ["cut"]);
}

#[test]
fn values_are_trimmed_and_cut_to_120_characters() {
    let root = Root::new("long");
    let id = "é".repeat(200);
    root.session(
        "p",
        "long.md",
        format!(
            "---\nid:    {id}   \ncreated:  {}  \nsession_id: {}\nstatus: open\n---\n",
            "9".repeat(200),
            "s".repeat(200)
        ),
    );
    let list = list_all(root.path());
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id.chars().count(), 120);
    assert_eq!(list[0].id, "é".repeat(120));
    assert_eq!(list[0].created.chars().count(), 120);
    assert_eq!(list[0].session_id.chars().count(), 120);
}

#[test]
fn a_crlf_file_parses() {
    let root = Root::new("crlf");
    root.session(
        "p",
        "crlf.md",
        "---\r\nid: crlf\r\ncreated: 2026-10-31T01:02:03\r\nstatus: open\r\n---\r\nbody\r\n",
    );
    let list = list_all(root.path());
    assert_eq!(ids(&list), ["crlf"]);
    assert_eq!(list[0].created, "2026-10-31T01:02:03");
}

#[test]
fn colons_in_values_and_unknown_keys_do_not_confuse_the_parse() {
    let root = Root::new("colons");
    root.session(
        "p",
        "colon.md",
        "---\nsession_id: pending (a: b)\nid: S-1: first\nunknown: ignored\nstatus: open\n---\n",
    );
    let list = list_all(root.path());
    assert_eq!(ids(&list), ["S-1: first"]);
    assert_eq!(list[0].session_id, "pending (a: b)");
}
