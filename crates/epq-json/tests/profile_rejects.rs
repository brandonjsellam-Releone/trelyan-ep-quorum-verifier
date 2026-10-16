//! The EP canonicalization profile (receipts-12 section 13.12): each hazard
//! class must be rejected, and boundary values on the accepting side must
//! still parse, so that every reject test is shown to be able to pass.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use epq_json::{JsonError, MAX_DEPTH, Value, canonicalize_str, parse};

/// Builds a JSON backslash-u escape at runtime. Writing such escapes literally
/// in this file is unsafe: an editing tool once silently decoded them into
/// plain characters, which made the escape tests vacuous.
fn esc(hex: &str) -> String {
    format!("{}u{hex}", '\\')
}

#[test]
fn escape_helper_produces_a_real_escape() {
    let e = esc("0061");
    assert_eq!(e.len(), 6);
    assert_eq!(e.as_bytes()[0], b'\\');
    assert_eq!(
        parse(&format!("\"{e}\"")).unwrap(),
        Value::String("a".to_owned())
    );
}

#[test]
fn duplicate_names_rejected_even_after_escape_decoding() {
    assert!(matches!(
        parse(r#"{"a":1,"a":2}"#),
        Err(JsonError::DuplicateName(_))
    ));
    let escaped_dup = format!(r#"{{"a":1,"{}":2}}"#, esc("0061"));
    assert!(
        escaped_dup.contains('\\'),
        "the escape must be present in the input"
    );
    assert!(matches!(
        parse(&escaped_dup),
        Err(JsonError::DuplicateName(_))
    ));
    // Accepting side: a different escaped name is not a duplicate.
    assert!(parse(&format!(r#"{{"a":1,"{}":2}}"#, esc("0062"))).is_ok());
    assert!(matches!(
        parse(r#"{"x":{"k":1,"k":1}}"#),
        Err(JsonError::DuplicateName(_))
    ));
    // Accepting side: the same name in sibling objects is fine.
    assert!(parse(r#"[{"a":1},{"a":2}]"#).is_ok());
}

#[test]
fn unpaired_surrogates_rejected() {
    let cases = [
        format!("\"{}\"", esc("d800")),
        format!("\"{}\"", esc("dc00")),
        format!("\"{}{}\"", esc("d800"), esc("0041")),
        format!("\"{} x\"", esc("d83d")),
        format!("\"{}{}\"", esc("de00"), esc("d83d")),
    ];
    for s in &cases {
        assert!(
            matches!(parse(s), Err(JsonError::UnpairedSurrogate(_))),
            "{s}"
        );
    }
    // Accepting side: a proper escaped pair decodes to one code point.
    let pair = format!("\"{}{}\"", esc("d83d"), esc("de00"));
    assert_eq!(
        pair.len(),
        14,
        "pair input must be two 6-byte escapes plus quotes"
    );
    assert_eq!(parse(&pair).unwrap(), Value::String("\u{1F600}".to_owned()));
    assert_eq!(
        canonicalize_str(&pair).unwrap().as_bytes(),
        b"\"\xf0\x9f\x98\x80\""
    );
}

#[test]
fn non_integers_rejected_including_integral_values_written_with_fraction_or_exponent() {
    for s in ["1.0", "0.5", "1e2", "1E2", "-1E-2", "2e+3", "[1,2.5]"] {
        assert!(matches!(parse(s), Err(JsonError::NonInteger(_))), "{s}");
    }
}

#[test]
fn integer_bounds() {
    assert_eq!(
        parse("9007199254740991").unwrap(),
        Value::Int(9_007_199_254_740_991)
    );
    assert_eq!(
        parse("-9007199254740991").unwrap(),
        Value::Int(-9_007_199_254_740_991)
    );
    for s in [
        "9007199254740992",
        "-9007199254740992",
        "99999999999999999999999",
    ] {
        assert!(
            matches!(parse(s), Err(JsonError::IntegerOutOfRange(_))),
            "{s}"
        );
    }
    assert_eq!(canonicalize_str("-0").unwrap(), "0");
}

#[test]
fn depth_limit_is_64() {
    let ok = format!("{}{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
    assert!(parse(&ok).is_ok());
    let deep = format!("{}{}", "[".repeat(MAX_DEPTH + 1), "]".repeat(MAX_DEPTH + 1));
    assert!(matches!(parse(&deep), Err(JsonError::DepthExceeded(_))));
    let mixed = format!(
        "{}1{}",
        r#"{"a":"#.repeat(MAX_DEPTH + 1),
        "}".repeat(MAX_DEPTH + 1)
    );
    assert!(matches!(parse(&mixed), Err(JsonError::DepthExceeded(_))));
}

#[test]
fn syntax_errors_rejected() {
    assert_eq!(parse(""), Err(JsonError::UnexpectedEnd));
    assert!(matches!(parse("{} x"), Err(JsonError::TrailingData(_))));
    assert!(matches!(parse("01"), Err(JsonError::Syntax(_))));
    assert!(matches!(
        parse("\"a\u{1}b\""),
        Err(JsonError::ControlCharacter(_))
    ));
    assert!(matches!(parse(r#""\x""#), Err(JsonError::InvalidEscape(_))));
    assert!(matches!(
        parse(r#""\u12G4""#),
        Err(JsonError::InvalidEscape(_))
    ));
    assert!(matches!(parse("[1,]"), Err(JsonError::Syntax(_))));
    assert!(matches!(parse(r#"{"a" 1}"#), Err(JsonError::Syntax(_))));
    assert!(matches!(parse("tru"), Err(JsonError::UnexpectedEnd)));
    assert!(matches!(parse("[1"), Err(JsonError::UnexpectedEnd)));
    assert!(matches!(parse("-"), Err(JsonError::UnexpectedEnd)));
}

#[test]
fn control_characters_escaped_per_rfc8785() {
    assert_eq!(
        canonicalize_str(r#""\u0008\u0009\u000a\u000c\u000d\u001f\u0001""#).unwrap(),
        r#""\b\t\n\f\r\u001f\u0001""#
    );
    // U+007F and '/' are written as-is.
    assert_eq!(canonicalize_str(r#""\u007f\/""#).unwrap(), "\"\u{7f}/\"");
}
