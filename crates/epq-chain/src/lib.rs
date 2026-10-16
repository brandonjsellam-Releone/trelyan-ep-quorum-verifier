//! The completed-signoff link of draft-schrock-ep-quorum-04, section 4:
//!
//! ```text
//! D = UTF8("EP-QUORUM-SIGNOFF-CHAIN-v1") || 0x00
//! link(S) = lowercase_hex(SHA-256(D || UTF8(JCS(S))))
//! ```
//!
//! `S` is the complete predecessor signoff object, including its signed
//! context and native proof fields. The separator is one zero octet, not a
//! printable backslash followed by `0`.

#![forbid(unsafe_code)]

use epq_json::{Value, canonicalize};
use sha2::{Digest, Sha256};

/// The profile label, without the separator.
pub const PROFILE_LABEL: &str = "EP-QUORUM-SIGNOFF-CHAIN-v1";

/// Domain prefix: the profile label followed by exactly one zero octet.
#[must_use]
pub fn domain() -> Vec<u8> {
    let mut d = PROFILE_LABEL.as_bytes().to_vec();
    d.push(0);
    d
}

/// Computes `link(S)` over the complete predecessor signoff `S`.
#[must_use]
pub fn link(signoff: &Value) -> String {
    let mut h = Sha256::new();
    h.update(domain());
    h.update(canonicalize(signoff).as_bytes());
    lower_hex(&h.finalize())
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from(HEX[usize::from(b >> 4)]));
        s.push(char::from(HEX[usize::from(b & 0x0f)]));
    }
    s
}
