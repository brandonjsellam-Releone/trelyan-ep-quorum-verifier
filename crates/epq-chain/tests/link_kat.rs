//! Known answers for `link(S)`, computed with an independent implementation
//! (GNU coreutils `sha256sum` over explicitly constructed bytes, checked
//! byte by byte with `od -c` on 2026-10-16), not with this crate.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use epq_chain::{PROFILE_LABEL, domain, link};
use epq_json::parse;

/// sha256("EP-QUORUM-SIGNOFF-CHAIN-v1" || 0x00 || r#"{"a":1}"#): 34 bytes.
const KAT_SMALL: &str = "00438347bff84ea2c38b4da2e4ca19093ea8f346969a6ebbb424d9aa992c4198";

/// The same input with a printable backslash and '0' instead of the zero octet
/// (35 bytes). The draft explicitly forbids this separator, so a correct
/// implementation must NOT produce it.
const WRONG_SEPARATOR: &str = "42b469aad3ccb1d5dab80aa30968ae6cbc6441ef0487083d75e1a3d0e444e0b6";

/// A signoff-shaped object, hashed over its canonical form.
const KAT_SIGNOFF: &str = "620094b2705531d95cb25cfa675d535143ab7342e99d00980030738a99b69255";

#[test]
fn domain_is_label_plus_one_zero_octet() {
    let d = domain();
    assert_eq!(d.len(), PROFILE_LABEL.len() + 1);
    assert_eq!(d.last(), Some(&0u8));
    assert_eq!(&d[..PROFILE_LABEL.len()], PROFILE_LABEL.as_bytes());
}

#[test]
fn link_matches_independent_kat() {
    let v = parse(r#"{"a":1}"#).unwrap();
    assert_eq!(link(&v), KAT_SMALL);
    assert_ne!(link(&v), WRONG_SEPARATOR);
}

#[test]
fn link_hashes_the_canonical_form_not_the_input_text() {
    // Same object as the KAT, deliberately non-canonical: shuffled keys and extra whitespace.
    let shuffled = r#"{
        "webauthn": {"signature": "CC", "client_data_json": "BB", "authenticator_data": "AA"},
        "context": {"issued_at": "2026-06-09T17:21:05Z", "context_type": "ep.signoff.v1",
                    "approver": "ep:approver:po_rivera", "action_hash": "sha256:9f2c"},
        "@type": "ep.signoff"
    }"#;
    assert_eq!(link(&parse(shuffled).unwrap()), KAT_SIGNOFF);
}

#[test]
fn any_change_to_the_signoff_changes_the_link() {
    let base = r#"{"@type":"ep.signoff","context":{"action_hash":"sha256:9f2c","approver":"ep:approver:po_rivera","context_type":"ep.signoff.v1","issued_at":"2026-06-09T17:21:05Z"},"webauthn":{"authenticator_data":"AA","client_data_json":"BB","signature":"CC"}}"#;
    let same = link(&parse(base).unwrap());
    assert_eq!(same, KAT_SIGNOFF);
    // Substituting a different signature over the same context must change the link (draft section 4).
    let other_sig = base.replace("\"signature\":\"CC\"", "\"signature\":\"CD\"");
    assert_ne!(link(&parse(&other_sig).unwrap()), same);
    // Dropping a proof field must also change it ("hash the complete signoff").
    let dropped = base.replace(",\"signature\":\"CC\"", "");
    assert_ne!(link(&parse(&dropped).unwrap()), same);
}
