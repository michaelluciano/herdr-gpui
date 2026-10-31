use super::*;

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";

fn record(id: &str, parents: &str, author: &str, time: &str, refs: &str, subject: &str) -> Vec<u8> {
    format!("{id}\0{parents}\0{author}\0{time}\0{refs}\0{subject}\u{1e}\n").into_bytes()
}

#[test]
fn a_record_parses_into_its_fields() {
    let mut input = record(
        A,
        &format!("{B} {C}"),
        "Ada",
        "1700000000",
        "HEAD -> refs/heads/main, refs/remotes/origin/main, tag: refs/tags/v1, refs/stash",
        "Merge it",
    );
    input.extend(record(B, "", "Bob", "1699999999", "", "Root"));
    let commits = parse_log(&input).unwrap();
    assert_eq!(commits.len(), 2);
    let head = &commits[0];
    assert_eq!(head.id, A);
    assert_eq!(head.parents, [B, C]);
    assert_eq!(head.author, "Ada");
    assert_eq!(head.time, 1_700_000_000);
    assert_eq!(head.subject, "Merge it");
    assert_eq!(
        head.refs,
        [
            Ref::Head,
            Ref::LocalBranch("main".into()),
            Ref::RemoteBranch {
                remote: "origin".into(),
                name: "main".into()
            },
            Ref::Tag("v1".into()),
            Ref::Other,
        ]
    );
    assert!(commits[1].parents.is_empty());
    assert!(commits[1].refs.is_empty());
}

#[test]
fn a_detached_head_and_slashed_names_parse() {
    let input = record(
        A,
        "",
        "x",
        "1",
        "HEAD, refs/heads/feat/x, refs/remotes/up/feat/x",
        "s",
    );
    let commits = parse_log(&input).unwrap();
    assert_eq!(
        commits[0].refs,
        [
            Ref::Head,
            Ref::LocalBranch("feat/x".into()),
            Ref::RemoteBranch {
                remote: "up".into(),
                name: "feat/x".into()
            },
        ]
    );
}

#[test]
fn empty_output_is_an_empty_history() {
    assert!(parse_log(b"").unwrap().is_empty());
    assert!(parse_log(b"\n").unwrap().is_empty());
}

#[test]
fn long_subjects_and_authors_are_capped_on_a_char_boundary() {
    let subject = "é".repeat(5000);
    let input = record(A, "", &"n".repeat(5000), "1", "", &subject);
    let commit = &parse_log(&input).unwrap()[0];
    assert!(commit.subject.len() <= 1024 && !commit.subject.is_empty());
    assert!(commit.subject.chars().all(|c| c == 'é'));
    assert!(commit.author.len() <= 1024);
}

#[test]
fn invalid_utf8_text_is_replaced_not_rejected() {
    let mut input = record(A, "", "x", "1", "", "ok");
    let at = input.len() - 4;
    input[at] = 0xff;
    assert!(parse_log(&input).is_ok());
}

#[test]
fn malformed_records_are_typed_errors() {
    let bad_id = record("zz", "", "x", "1", "", "s");
    assert!(matches!(
        parse_log(&bad_id),
        Err(Error::Malformed {
            index: 0,
            field: Field::Id
        })
    ));
    let bad_parent = record(A, "nothex", "x", "1", "", "s");
    assert!(matches!(
        parse_log(&bad_parent),
        Err(Error::Malformed {
            field: Field::Parent,
            ..
        })
    ));
    let bad_time = record(A, "", "x", "soon", "", "s");
    assert!(matches!(
        parse_log(&bad_time),
        Err(Error::Malformed {
            field: Field::Time,
            ..
        })
    ));
    let few_fields = b"abc\0def\x1e".to_vec();
    assert!(matches!(
        parse_log(&few_fields),
        Err(Error::Malformed {
            field: Field::Shape,
            ..
        })
    ));
    let mut second = record(A, "", "x", "1", "", "s");
    second.extend(record("short", "", "x", "1", "", "s"));
    assert!(matches!(
        parse_log(&second),
        Err(Error::Malformed {
            index: 1,
            field: Field::Id
        })
    ));
}

#[test]
fn an_unterminated_record_is_truncated() {
    let mut input = record(A, "", "x", "1", "", "s");
    input.extend_from_slice(format!("{B}\0\0x\01\0\0partial").as_bytes());
    assert!(matches!(parse_log(&input), Err(Error::Truncated)));
}

#[test]
fn input_and_history_are_bounded() {
    let big = vec![b'x'; MAX_INPUT + 1];
    assert!(matches!(parse_log(&big), Err(Error::TooLarge { .. })));
    let one = record(A, "", "x", "1", "", "s");
    let many: Vec<u8> = one.repeat(MAX_COMMITS + 1);
    assert!(matches!(parse_log(&many), Err(Error::TooMany { .. })));
}

#[test]
fn garbage_never_panics() {
    let valid = record(A, B, "Ada", "1700000000", "HEAD -> refs/heads/m", "s");
    let mut inputs: Vec<Vec<u8>> = vec![
        vec![0x1e],
        vec![0],
        vec![0x1e; 50],
        vec![0; 50],
        b"\0\0\0\0\0\x1e".to_vec(),
        b"\0\0\0\0\0\0\x1e".to_vec(),
        vec![0xff, 0xfe, 0x00, 0x1e],
        "\u{1e}\0 \u{1e}".as_bytes().to_vec(),
    ];
    for cut in 0..valid.len() {
        inputs.push(valid[..cut].to_vec());
    }
    for at in 0..valid.len() {
        let mut flipped = valid.clone();
        flipped[at] = flipped[at].wrapping_add(0x7f);
        inputs.push(flipped);
    }
    for input in &inputs {
        let _ = parse_log(input);
    }
}

#[test]
fn ref_names_that_are_not_branches_or_tags_are_other() {
    let input = record(
        A,
        "",
        "x",
        "1",
        "refs/notes/x, refs/remotes/onlyremote, refs/heads/",
        "s",
    );
    let refs = &parse_log(&input).unwrap()[0].refs;
    assert_eq!(refs.len(), 3);
    assert!(refs.iter().all(|r| *r == Ref::Other));
}
