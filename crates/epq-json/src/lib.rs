//! Strict JSON parse boundary and RFC 8785 (JCS) canonical serialization.
//!
//! The accepted profile follows draft-schrock-ep-authorization-receipts-12,
//! section 13.12 ("Canonicalization Robustness"). The parser rejects:
//! - duplicate object member names, compared after escape decoding;
//! - unpaired UTF-16 surrogate escapes;
//! - numbers that are not integers, or integers with magnitude above 2^53 - 1;
//! - container nesting deeper than 64.
//!
//! Lexical rule used here: a number is accepted only when written as an
//! optional minus sign followed by digits. A number written with a fraction or
//! an exponent (for example `1.0` or `1e2`) is rejected, even when its value is
//! integral. `-0` is accepted and canonicalized to `0`.
//!
//! Every accepted value is an integer, so JCS number serialization reduces to
//! plain decimal integer formatting. No floating-point formatting is involved.

#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::collections::BTreeSet;

/// Maximum container nesting depth accepted by the profile.
pub const MAX_DEPTH: usize = 64;

/// Largest integer magnitude accepted by the profile: 2^53 - 1.
pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// A parsed JSON value within the EP canonicalization profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
    Array(Vec<Self>),
    /// Members in input order. [`canonicalize`] sorts them.
    Object(Vec<(String, Self)>),
}

impl Value {
    /// Returns the member named `key` if this value is an object.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

/// Why an input was rejected. Positions are byte offsets into the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum JsonError {
    #[error("unexpected end of input")]
    UnexpectedEnd,
    #[error("syntax error at byte {0}")]
    Syntax(usize),
    #[error("trailing data at byte {0}")]
    TrailingData(usize),
    #[error("duplicate member name at byte {0}")]
    DuplicateName(usize),
    #[error("unpaired UTF-16 surrogate escape at byte {0}")]
    UnpairedSurrogate(usize),
    #[error("invalid escape sequence at byte {0}")]
    InvalidEscape(usize),
    #[error("unescaped control character in string at byte {0}")]
    ControlCharacter(usize),
    #[error("non-integer number at byte {0}")]
    NonInteger(usize),
    #[error("integer magnitude above 2^53-1 at byte {0}")]
    IntegerOutOfRange(usize),
    #[error("nesting deeper than 64 at byte {0}")]
    DepthExceeded(usize),
}

/// Parses `input` under the strict profile.
///
/// # Errors
/// Returns a [`JsonError`] for any input outside the profile. Nothing is
/// silently repaired.
pub fn parse(input: &str) -> Result<Value, JsonError> {
    let mut p = Parser {
        bytes: input.as_bytes(),
        pos: 0,
    };
    p.skip_ws();
    let v = p.value(0)?;
    p.skip_ws();
    if p.pos == p.bytes.len() {
        Ok(v)
    } else {
        Err(JsonError::TrailingData(p.pos))
    }
}

/// Serializes `value` in RFC 8785 canonical form.
#[must_use]
pub fn canonicalize(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value);
    out
}

/// Parses `input` and returns its canonical form.
///
/// # Errors
/// Returns a [`JsonError`] if `input` is outside the profile.
pub fn canonicalize_str(input: &str) -> Result<String, JsonError> {
    parse(input).map(|v| canonicalize(&v))
}

/// RFC 8785 section 3.2.3: compare property names as sequences of UTF-16 code units.
fn utf16_order(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Int(n) => out.push_str(&n.to_string()),
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Value::Object(members) => {
            let mut sorted: Vec<&(String, Value)> = members.iter().collect();
            sorted.sort_by(|(a, _), (b, _)| utf16_order(a, b));
            out.push('{');
            for (i, (k, v)) in sorted.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(out, k);
                out.push(':');
                write_value(out, v);
            }
            out.push('}');
        }
    }
}

/// RFC 8785 section 3.2.2.2 string serialization.
fn write_string(out: &mut String, s: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                let n = c as u32;
                out.push_str("\\u00");
                out.push(char::from(HEX[((n >> 4) & 0xf) as usize]));
                out.push(char::from(HEX[(n & 0xf) as usize]));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect_literal(&mut self, lit: &[u8], v: Value) -> Result<Value, JsonError> {
        let end = self.pos + lit.len();
        match self.bytes.get(self.pos..end) {
            Some(s) if s == lit => {
                self.pos = end;
                Ok(v)
            }
            Some(_) => Err(JsonError::Syntax(self.pos)),
            None => Err(JsonError::UnexpectedEnd),
        }
    }

    fn value(&mut self, depth: usize) -> Result<Value, JsonError> {
        match self.peek() {
            None => Err(JsonError::UnexpectedEnd),
            Some(b'{') => self.object(depth + 1),
            Some(b'[') => self.array(depth + 1),
            Some(b'"') => self.string().map(Value::String),
            Some(b't') => self.expect_literal(b"true", Value::Bool(true)),
            Some(b'f') => self.expect_literal(b"false", Value::Bool(false)),
            Some(b'n') => self.expect_literal(b"null", Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(JsonError::Syntax(self.pos)),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Value, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::DepthExceeded(self.pos));
        }
        self.pos += 1; // '{'
        let mut members = Vec::new();
        let mut seen = BTreeSet::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Object(members));
        }
        loop {
            self.skip_ws();
            let key_pos = self.pos;
            if self.peek() != Some(b'"') {
                return Err(self.syntax_or_end());
            }
            let key = self.string()?;
            if !seen.insert(key.clone()) {
                return Err(JsonError::DuplicateName(key_pos));
            }
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(self.syntax_or_end());
            }
            self.pos += 1;
            self.skip_ws();
            let v = self.value(depth)?;
            members.push((key, v));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Value::Object(members));
                }
                _ => return Err(self.syntax_or_end()),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Value, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::DepthExceeded(self.pos));
        }
        self.pos += 1; // '['
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value(depth)?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Value::Array(items));
                }
                _ => return Err(self.syntax_or_end()),
            }
        }
    }

    const fn syntax_or_end(&self) -> JsonError {
        if self.pos >= self.bytes.len() {
            JsonError::UnexpectedEnd
        } else {
            JsonError::Syntax(self.pos)
        }
    }

    fn hex4(&mut self) -> Result<u16, JsonError> {
        let start = self.pos;
        let digits = self
            .bytes
            .get(start..start + 4)
            .ok_or(JsonError::UnexpectedEnd)?;
        let mut n: u16 = 0;
        for &d in digits {
            let v = match d {
                b'0'..=b'9' => d - b'0',
                b'a'..=b'f' => d - b'a' + 10,
                b'A'..=b'F' => d - b'A' + 10,
                _ => return Err(JsonError::InvalidEscape(start)),
            };
            n = (n << 4) | u16::from(v);
        }
        self.pos += 4;
        Ok(n)
    }

    fn string(&mut self) -> Result<String, JsonError> {
        self.pos += 1; // opening quote
        let mut out = String::new();
        loop {
            let start = self.pos;
            let Some(b) = self.peek() else {
                return Err(JsonError::UnexpectedEnd);
            };
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let Some(e) = self.peek() else {
                        return Err(JsonError::UnexpectedEnd);
                    };
                    self.pos += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(self.unicode_escape(start)?),
                        _ => return Err(JsonError::InvalidEscape(start)),
                    }
                }
                0x00..=0x1f => return Err(JsonError::ControlCharacter(start)),
                _ => {
                    // The input is valid UTF-8 (it came from a &str), so copy one whole code point.
                    let len = utf8_len(b);
                    let chunk = self
                        .bytes
                        .get(start..start + len)
                        .ok_or(JsonError::UnexpectedEnd)?;
                    let s = std::str::from_utf8(chunk).map_err(|_| JsonError::Syntax(start))?;
                    out.push_str(s);
                    self.pos += len;
                }
            }
        }
    }

    /// Decodes `\uXXXX`, pairing surrogates. `esc_start` is the position of the backslash.
    fn unicode_escape(&mut self, esc_start: usize) -> Result<char, JsonError> {
        let hi = self.hex4()?;
        match hi {
            0xD800..=0xDBFF => {
                if self.bytes.get(self.pos..self.pos + 2) != Some(b"\\u") {
                    return Err(JsonError::UnpairedSurrogate(esc_start));
                }
                self.pos += 2;
                let lo = self.hex4()?;
                if !(0xDC00..=0xDFFF).contains(&lo) {
                    return Err(JsonError::UnpairedSurrogate(esc_start));
                }
                let cp = 0x10000 + ((u32::from(hi) - 0xD800) << 10) + (u32::from(lo) - 0xDC00);
                char::from_u32(cp).ok_or(JsonError::UnpairedSurrogate(esc_start))
            }
            0xDC00..=0xDFFF => Err(JsonError::UnpairedSurrogate(esc_start)),
            _ => char::from_u32(u32::from(hi)).ok_or(JsonError::InvalidEscape(esc_start)),
        }
    }

    fn number(&mut self) -> Result<Value, JsonError> {
        let start = self.pos;
        let negative = self.peek() == Some(b'-');
        if negative {
            self.pos += 1;
        }
        let digits_start = self.pos;
        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(JsonError::Syntax(self.pos)); // leading zero
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            Some(_) => return Err(JsonError::Syntax(self.pos)),
            None => return Err(JsonError::UnexpectedEnd),
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(JsonError::NonInteger(start));
        }
        let mut magnitude: i64 = 0;
        for &d in &self.bytes[digits_start..self.pos] {
            magnitude = magnitude
                .checked_mul(10)
                .and_then(|m| m.checked_add(i64::from(d - b'0')))
                .ok_or(JsonError::IntegerOutOfRange(start))?;
            if magnitude > MAX_SAFE_INTEGER {
                return Err(JsonError::IntegerOutOfRange(start));
            }
        }
        Ok(Value::Int(if negative { -magnitude } else { magnitude }))
    }
}

const fn utf8_len(first: u8) -> usize {
    match first {
        0xF0..=0xF7 => 4,
        0xE0..=0xEF => 3,
        0xC0..=0xDF => 2,
        _ => 1,
    }
}
