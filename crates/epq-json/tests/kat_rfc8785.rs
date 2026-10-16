//! Known-answer tests taken from RFC 8785 (JSON Canonicalization Scheme).
//!
//! Source: rfc8785.txt, sha256 recorded in the evidence SHA256SUMS. The RFC's
//! main sample also contains non-integer numbers, which this profile rejects
//! by design. The byte KAT therefore uses the RFC's own expected bytes with
//! the "numbers" member removed, and a separate test checks that the full
//! sample is rejected.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use epq_json::{JsonError, Value, canonicalize, canonicalize_str, parse};

fn hex_decode(s: &str) -> Vec<u8> {
    let digits: Vec<u8> = s.bytes().filter(u8::is_ascii_hexdigit).collect();
    digits
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

/// RFC 8785 section 3.2.3: property sorting by UTF-16 code units.
#[test]
fn rfc8785_sorting_kat() {
    let input = r#"{
       "\u20ac": "Euro Sign",
       "\r": "Carriage Return",
       "\ufb33": "Hebrew Letter Dalet With Dagesh",
       "1": "One",
       "\ud83d\ude00": "Emoji: Grinning Face",
       "\u0080": "Control",
       "\u00f6": "Latin Small Letter O With Diaeresis"
     }"#;
    let canonical = canonicalize_str(input).unwrap();
    let Value::Object(members) = parse(&canonical).unwrap() else {
        panic!("not an object")
    };
    let order: Vec<&str> = members
        .iter()
        .map(|(_, v)| match v {
            Value::String(s) => s.as_str(),
            _ => panic!("value not a string"),
        })
        .collect();
    assert_eq!(
        order,
        [
            "Carriage Return",
            "One",
            "Control",
            "Latin Small Letter O With Diaeresis",
            "Euro Sign",
            "Emoji: Grinning Face",
            "Hebrew Letter Dalet With Dagesh",
        ]
    );
}

/// RFC 8785 sections 3.2.2 and 3.2.4: string serialization and UTF-8 bytes,
/// with the sample's non-integer "numbers" member removed.
#[test]
fn rfc8785_string_and_bytes_kat() {
    let input = r#"{
       "string": "\u20ac$\u000F\u000aA'\u0042\u0022\u005c\\\"\/",
       "literals": [null, true, false]
     }"#;
    let canonical = canonicalize_str(input).unwrap();
    assert_eq!(
        canonical,
        r#"{"literals":[null,true,false],"string":"€$\u000f\nA'B\"\\\\\"/"}"#
    );
    // The RFC 8785 section 3.2.4 byte listing, minus the bytes of `"numbers":[...],`.
    let expected = hex_decode(
        "7b 22 6c 69 74 65 72 61 6c 73 22 3a 5b 6e 75 6c 6c 2c 74 72 \
         75 65 2c 66 61 6c 73 65 5d 2c 22 73 74 72 69 6e 67 22 3a 22 \
         e2 82 ac 24 5c 75 30 30 30 66 5c 6e 41 27 42 5c 22 5c 5c 5c \
         5c 5c 22 2f 22 7d",
    );
    assert_eq!(canonical.as_bytes(), expected.as_slice());
}

/// The complete RFC 8785 sample contains fractions and exponents. Under the
/// EP profile it must be rejected, not rounded.
#[test]
fn rfc8785_full_sample_rejected_by_profile() {
    let input = r#"{
       "numbers": [333333333.33333329, 1E30, 4.50,
                   2e-3, 0.000000000000000000000000001],
       "string": "\u20ac$\u000F\u000aA'\u0042\u0022\u005c\\\"\/",
       "literals": [null, true, false]
     }"#;
    assert!(matches!(parse(input), Err(JsonError::NonInteger(_))));
}

/// The canonical form is a fixed point.
#[test]
fn canonical_form_is_idempotent() {
    let input = r#"{"b":[1,{"z":null,"a":"\u00e9"}],"a":-0,"\u0000":"x\u007f"}"#;
    let once = canonicalize_str(input).unwrap();
    let twice = canonicalize(&parse(&once).unwrap());
    assert_eq!(once, twice);
    assert_eq!(
        once,
        "{\"\\u0000\":\"x\u{7f}\",\"a\":0,\"b\":[1,{\"a\":\"é\",\"z\":null}]}"
    );
}
