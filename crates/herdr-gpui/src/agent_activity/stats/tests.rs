use super::*;

#[test]
fn test_format_context() {
    assert_eq!(format_context(115_300), "115k");
    assert_eq!(format_context(10_000), "10k");
    assert_eq!(format_context(14_999), "10k");
    assert_eq!(format_context(5_000), "5k");
    assert_eq!(format_context(4_999), "<5k");
    assert_eq!(format_context(0), "<5k");
    assert_eq!(format_context(1_234_567), "1230k");
}

#[test]
fn test_context_band() {
    assert_eq!(context_band(0, None), ContextBand::Normal);
    assert_eq!(context_band(100, Some(0)), ContextBand::Normal);

    let limit = 250_000;
    assert_eq!(context_band(174_999, Some(limit)), ContextBand::Normal);
    assert_eq!(context_band(175_000, Some(limit)), ContextBand::Warm);
    assert_eq!(context_band(224_999, Some(limit)), ContextBand::Warm);
    assert_eq!(context_band(225_000, Some(limit)), ContextBand::Alert);
    assert_eq!(context_band(400_000, Some(limit)), ContextBand::Alert);
}

#[test]
fn test_model_letter() {
    assert_eq!(model_letter("haiku"), Some('H'));
    assert_eq!(model_letter("sonnet"), Some('S'));
    assert_eq!(model_letter("opus"), Some('O'));
    assert_eq!(model_letter("fable"), Some('F'));
    assert_eq!(model_letter("claude-sonnet-5-5"), Some('S'));
    assert_eq!(model_letter("Opus 5.5"), Some('O'));
    assert_eq!(model_letter("gpt-5"), None);
    assert_eq!(model_letter(""), None);
}

#[test]
fn test_parse_transcript_tail_last_qualifying_wins() {
    let transcript = r#"{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":2,"cache_read_input_tokens":3,"output_tokens":4}}}
{"type":"user","message":{"content":"hello"}}
{"type":"assistant","message":{"model":"<synthetic>","usage":{"input_tokens":5,"output_tokens":6}}}
{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":20,"cache_read_input_tokens":30,"output_tokens":40}}}"#;

    let expected = Stats {
        model: "claude-sonnet-5-5".to_string(),
        context: 10 + 20 + 30 + 40,
    };
    assert_eq!(parse_transcript_tail(transcript), Some(expected));
}

#[test]
fn test_parse_transcript_tail_truncated_first_line_skipped() {
    let transcript = r#"{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":2,"cache_read_input_tokens":3,"output_tokens":4}}
{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":20,"cache_read_input_tokens":30,"output_tokens":40}}}"#;

    let expected = Stats {
        model: "claude-sonnet-5-5".to_string(),
        context: 10 + 20 + 30 + 40,
    };
    assert_eq!(parse_transcript_tail(transcript), Some(expected));
}

#[test]
fn test_parse_transcript_tail_user_and_synthetic_dont_hide_real() {
    let transcript = r#"{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":20,"cache_read_input_tokens":30,"output_tokens":40}}}
{"type":"user","message":{"content":"hello"}}
{"type":"assistant","message":{"model":"<synthetic>","usage":{"input_tokens":5,"output_tokens":6}}}"#;

    let expected = Stats {
        model: "claude-sonnet-5-5".to_string(),
        context: 10 + 20 + 30 + 40,
    };
    assert_eq!(parse_transcript_tail(transcript), Some(expected));
}

#[test]
fn test_parse_transcript_tail_empty_string() {
    assert_eq!(parse_transcript_tail(""), None);
}

#[test]
fn test_parse_transcript_tail_missing_usage_fields() {
    let transcript = r#"{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":5,"output_tokens":10}}}"#;
    let expected = Stats {
        model: "claude-sonnet-5-5".to_string(),
        context: 15,
    };
    assert_eq!(parse_transcript_tail(transcript), Some(expected));
}

#[test]
fn test_parse_transcript_tail_no_qualifying_line() {
    let transcript = r#"{"type":"assistant","message":{"model":"<synthetic>","usage":{"input_tokens":5,"output_tokens":10}}}
{"type":"assistant","message":{"model":"gpt-5"}}"#;
    assert_eq!(parse_transcript_tail(transcript), None);
}

#[test]
fn test_parse_transcript_tail_non_json_line_skipped() {
    let transcript = r#"not a json
{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":2,"cache_read_input_tokens":3,"output_tokens":4}}}"#;
    let expected = Stats {
        model: "claude-sonnet-5-5".to_string(),
        context: 1 + 2 + 3 + 4,
    };
    assert_eq!(parse_transcript_tail(transcript), Some(expected));
}

#[test]
fn test_parse_transcript_tail_non_assistant_type() {
    let transcript = r#"{"type":"user","message":{"content":"hello"}}
{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":2,"cache_read_input_tokens":3,"output_tokens":4}}}"#;
    let expected = Stats {
        model: "claude-sonnet-5-5".to_string(),
        context: 1 + 2 + 3 + 4,
    };
    assert_eq!(parse_transcript_tail(transcript), Some(expected));
}

#[test]
fn test_parse_transcript_tail_assistant_without_usage() {
    let transcript = r#"{"type":"assistant","message":{"model":"claude-sonnet-5-5"}}"#;
    assert_eq!(parse_transcript_tail(transcript), None);
}

#[test]
fn test_parse_transcript_tail_assistant_missing_usage_fields() {
    let transcript = r#"{"type":"assistant","message":{"model":"claude-sonnet-5-5","usage":{"input_tokens":5}}}"#;
    let expected = Stats {
        model: "claude-sonnet-5-5".to_string(),
        context: 5,
    };
    assert_eq!(parse_transcript_tail(transcript), Some(expected));
}
