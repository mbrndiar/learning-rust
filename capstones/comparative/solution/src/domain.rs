//! Storage-independent values and restricted JSON parsing.

use crate::KvError;
use serde_json::{Map, Number, Value};
use std::collections::HashMap;

pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub const MAX_VALUE_BYTES: usize = 65_536;
pub const MAX_CONTAINER_DEPTH: usize = 32;

/// A validated key from the shared ASCII key grammar.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(String);

impl Key {
    /// Parses and validates a key.
    pub fn parse(value: &str) -> Result<Self, KvError> {
        let bytes = value.as_bytes();
        let first_is_alphanumeric = bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_alphanumeric());
        let remainder_is_valid = bytes[bytes.len().min(1)..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'));

        if bytes.len() > 128 || !value.is_ascii() || !first_is_alphanumeric || !remainder_is_valid {
            return Err(KvError::InvalidArgument {
                field: "key",
                reason: "format",
            });
        }

        Ok(Self(value.to_owned()))
    }

    /// Borrows the validated key text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A positive JSON-safe global revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(u64);

impl Revision {
    /// Validates a positive revision in the shared safe-integer range.
    pub fn new(value: u64) -> Result<Self, KvError> {
        if !(1..=MAX_SAFE_INTEGER).contains(&value) {
            return Err(KvError::InvalidArgument {
                field: "expect",
                reason: "format",
            });
        }
        Ok(Self(value))
    }

    /// Returns the numeric revision.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Conditional behavior supported by `set`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetExpectation {
    Any,
    Absent,
    Exact(Revision),
}

/// Conditional behavior supported by `delete`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteExpectation {
    Any,
    Exact(Revision),
}

/// One live key/value entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: Key,
    pub value: Value,
    pub revision: Revision,
}

/// Successful `set` outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct SetResult {
    pub entry: Entry,
    pub created: bool,
}

/// Successful `delete` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteResult {
    pub deleted_revision: Revision,
    pub revision: Revision,
}

/// Successful `list` outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct ListResult {
    pub entries: Vec<Entry>,
    pub global_revision: u64,
}

/// Storage-independent command accepted by [`crate::KvApplication`].
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Set {
        key: Key,
        value: Value,
        expectation: SetExpectation,
    },
    Get {
        key: Key,
    },
    Delete {
        key: Key,
        expectation: DeleteExpectation,
    },
    List,
}

/// Storage-independent successful command result.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    Set(Box<SetResult>),
    Get(Box<Entry>),
    Delete(DeleteResult),
    List(ListResult),
}

/// Parses, validates, and normalizes one restricted JSON value.
pub fn parse_json_value(input: &str) -> Result<Value, KvError> {
    parse_json(input, false)
}

pub(crate) fn parse_stored_json(input: &str) -> Result<Value, KvError> {
    parse_json(input, true)
}

fn parse_json(input: &str, require_normalized: bool) -> Result<Value, KvError> {
    if input.len() > MAX_VALUE_BYTES {
        return Err(KvError::InvalidValue {
            reason: "byte_limit",
        });
    }

    let mut parser = JsonParser::new(input);
    let raw = parser.parse_value(0)?;
    parser.skip_whitespace();
    if !parser.is_finished() {
        return Err(KvError::InvalidJson);
    }
    let mut metadata = ValidationMetadata {
        normalized: !parser.saw_whitespace,
    };
    let value = normalize_value(raw, &mut metadata, 0)?;
    if require_normalized && !metadata.normalized {
        return Err(KvError::InvalidValue {
            reason: "not_normalized",
        });
    }
    Ok(value)
}

#[derive(Debug)]
enum RawValue {
    Null,
    Bool(bool),
    String(RawString),
    Number(String),
    Array(Vec<Self>),
    Object(Vec<(RawString, Self)>),
}

#[derive(Debug)]
struct RawString {
    value: String,
    unpaired_surrogate: bool,
}

struct JsonParser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    position: usize,
    saw_whitespace: bool,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            position: 0,
            saw_whitespace: false,
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<RawValue, KvError> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'n') => {
                self.consume_literal(b"null")?;
                Ok(RawValue::Null)
            }
            Some(b't') => {
                self.consume_literal(b"true")?;
                Ok(RawValue::Bool(true))
            }
            Some(b'f') => {
                self.consume_literal(b"false")?;
                Ok(RawValue::Bool(false))
            }
            Some(b'"') => self.parse_string().map(RawValue::String),
            Some(b'[') => self.parse_array(depth + 1),
            Some(b'{') => self.parse_object(depth + 1),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(RawValue::Number),
            _ => Err(KvError::InvalidJson),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<RawValue, KvError> {
        self.position += 1;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.consume_if(b']') {
            return Ok(RawValue::Array(values));
        }

        loop {
            values.push(self.parse_value(depth)?);
            self.skip_whitespace();
            if self.consume_if(b']') {
                break;
            }
            if !self.consume_if(b',') {
                return Err(KvError::InvalidJson);
            }
        }
        Ok(RawValue::Array(values))
    }

    fn parse_object(&mut self, depth: usize) -> Result<RawValue, KvError> {
        self.position += 1;
        self.skip_whitespace();
        let mut members = Vec::new();
        if self.consume_if(b'}') {
            return Ok(RawValue::Object(members));
        }

        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(KvError::InvalidJson);
            }
            let name = self.parse_string()?;
            self.skip_whitespace();
            if !self.consume_if(b':') {
                return Err(KvError::InvalidJson);
            }
            let value = self.parse_value(depth)?;
            members.push((name, value));
            self.skip_whitespace();
            if self.consume_if(b'}') {
                break;
            }
            if !self.consume_if(b',') {
                return Err(KvError::InvalidJson);
            }
        }
        Ok(RawValue::Object(members))
    }

    fn parse_string(&mut self) -> Result<RawString, KvError> {
        self.position += 1;
        let mut output = String::new();
        let mut unpaired_surrogate = false;
        let mut segment_start = self.position;

        while let Some(byte) = self.peek() {
            match byte {
                b'"' => {
                    output.push_str(&self.input[segment_start..self.position]);
                    self.position += 1;
                    return Ok(RawString {
                        value: output,
                        unpaired_surrogate,
                    });
                }
                b'\\' => {
                    output.push_str(&self.input[segment_start..self.position]);
                    self.position += 1;
                    let escaped = self.next().ok_or(KvError::InvalidJson)?;
                    match escaped {
                        b'"' => output.push('"'),
                        b'\\' => output.push('\\'),
                        b'/' => output.push('/'),
                        b'b' => output.push('\u{0008}'),
                        b'f' => output.push('\u{000c}'),
                        b'n' => output.push('\n'),
                        b'r' => output.push('\r'),
                        b't' => output.push('\t'),
                        b'u' => {
                            self.parse_unicode_escape(&mut output, &mut unpaired_surrogate)?;
                        }
                        _ => return Err(KvError::InvalidJson),
                    }
                    segment_start = self.position;
                }
                0x00..=0x1f => return Err(KvError::InvalidJson),
                0x20..=0x7f => self.position += 1,
                _ => {
                    let character = self.input[self.position..]
                        .chars()
                        .next()
                        .ok_or(KvError::InvalidJson)?;
                    self.position += character.len_utf8();
                }
            }
        }
        Err(KvError::InvalidJson)
    }

    fn parse_unicode_escape(
        &mut self,
        output: &mut String,
        unpaired_surrogate: &mut bool,
    ) -> Result<(), KvError> {
        let first = self.parse_hex_quad()?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            if self.peek() != Some(b'\\')
                || self.bytes.get(self.position + 1).copied() != Some(b'u')
            {
                *unpaired_surrogate = true;
                output.push('\u{fffd}');
                return Ok(());
            }
            self.position += 2;
            let second = self.parse_hex_quad()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                *unpaired_surrogate = true;
                output.push('\u{fffd}');
                if !(0xd800..=0xdfff).contains(&second) {
                    output.push(char::from_u32(u32::from(second)).ok_or(KvError::InvalidJson)?);
                }
                return Ok(());
            }
            0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
        } else if (0xdc00..=0xdfff).contains(&first) {
            *unpaired_surrogate = true;
            output.push('\u{fffd}');
            return Ok(());
        } else {
            u32::from(first)
        };
        output.push(char::from_u32(scalar).ok_or(KvError::InvalidJson)?);
        Ok(())
    }

    fn parse_hex_quad(&mut self) -> Result<u16, KvError> {
        let end = self.position.checked_add(4).ok_or(KvError::InvalidJson)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(KvError::InvalidJson)?;
        let mut value = 0_u16;
        for byte in bytes {
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a' + 10),
                b'A'..=b'F' => u16::from(byte - b'A' + 10),
                _ => return Err(KvError::InvalidJson),
            };
            value = value * 16 + digit;
        }
        self.position = end;
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<String, KvError> {
        let start = self.position;
        self.consume_if(b'-');
        match self.peek() {
            Some(b'0') => {
                self.position += 1;
                if self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                    return Err(KvError::InvalidJson);
                }
            }
            Some(b'1'..=b'9') => {
                self.position += 1;
                while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                    self.position += 1;
                }
            }
            _ => return Err(KvError::InvalidJson),
        }

        if self.consume_if(b'.') {
            if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                return Err(KvError::InvalidJson);
            }
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.position += 1;
            }
        }

        if self.peek().is_some_and(|byte| matches!(byte, b'e' | b'E')) {
            self.position += 1;
            if self.peek().is_some_and(|byte| matches!(byte, b'+' | b'-')) {
                self.position += 1;
            }
            if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                return Err(KvError::InvalidJson);
            }
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.position += 1;
            }
        }

        Ok(self.input[start..self.position].to_owned())
    }

    fn consume_literal(&mut self, literal: &[u8]) -> Result<(), KvError> {
        if self.bytes.get(self.position..self.position + literal.len()) == Some(literal) {
            self.position += literal.len();
            Ok(())
        } else {
            Err(KvError::InvalidJson)
        }
    }

    fn skip_whitespace(&mut self) {
        while self
            .peek()
            .is_some_and(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.saw_whitespace = true;
            self.position += 1;
        }
    }

    fn consume_if(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.position += 1;
        Some(byte)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }
}

struct ValidationMetadata {
    normalized: bool,
}

fn normalize_value(
    raw: RawValue,
    metadata: &mut ValidationMetadata,
    depth: usize,
) -> Result<Value, KvError> {
    match raw {
        RawValue::Null => Ok(Value::Null),
        RawValue::Bool(value) => Ok(Value::Bool(value)),
        RawValue::String(value) => {
            validate_raw_string(&value)?;
            Ok(Value::String(value.value))
        }
        RawValue::Number(token) => {
            let integer = normalize_number(&token)?;
            if token != integer.to_string() {
                metadata.normalized = false;
            }
            Ok(Value::Number(Number::from(integer)))
        }
        RawValue::Array(values) => {
            let next_depth = checked_container_depth(depth)?;
            values
                .into_iter()
                .map(|value| normalize_value(value, metadata, next_depth))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array)
        }
        RawValue::Object(members) => {
            let next_depth = checked_container_depth(depth)?;
            let mut last_indices = HashMap::new();
            for (index, (name, _)) in members.iter().enumerate() {
                if last_indices.insert(name.value.clone(), index).is_some() {
                    metadata.normalized = false;
                }
            }

            let mut map = Map::new();
            for (index, (name, value)) in members.into_iter().enumerate() {
                if last_indices.get(name.value.as_str()) == Some(&index) {
                    validate_raw_string(&name)?;
                    map.insert(name.value, normalize_value(value, metadata, next_depth)?);
                }
            }
            Ok(Value::Object(map))
        }
    }
}

fn validate_raw_string(value: &RawString) -> Result<(), KvError> {
    if value.unpaired_surrogate {
        Err(KvError::InvalidValue {
            reason: "unpaired_surrogate",
        })
    } else {
        Ok(())
    }
}

fn checked_container_depth(depth: usize) -> Result<usize, KvError> {
    let depth = depth + 1;
    if depth > MAX_CONTAINER_DEPTH {
        Err(KvError::InvalidValue {
            reason: "depth_limit",
        })
    } else {
        Ok(depth)
    }
}

fn normalize_number(token: &str) -> Result<i64, KvError> {
    let binary64 = token.parse::<f64>().map_err(|_| KvError::InvalidJson)?;
    if !binary64.is_finite() {
        return Err(KvError::InvalidValue {
            reason: "non_finite_number",
        });
    }

    let unsigned = token.strip_prefix('-').unwrap_or(token);
    let (mantissa, exponent_text) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, None), |(left, right)| (left, Some(right)));
    let exponent = parse_saturating_exponent(exponent_text.unwrap_or("0"));
    let (integer_part, fraction) = mantissa
        .split_once('.')
        .map_or((mantissa, ""), |(left, right)| (left, right));
    let mut digits = String::with_capacity(integer_part.len() + fraction.len());
    digits.push_str(integer_part);
    digits.push_str(fraction);
    if digits.bytes().all(|byte| byte == b'0') {
        return Ok(0);
    }
    let scale = i64::try_from(fraction.len())
        .unwrap_or(i64::MAX)
        .saturating_sub(exponent);

    let integer_digits = if scale <= 0 {
        let zero_count = usize::try_from(scale.saturating_neg()).unwrap_or(usize::MAX);
        if digits
            .trim_start_matches('0')
            .len()
            .saturating_add(zero_count)
            > 16
        {
            return Err(KvError::InvalidValue {
                reason: "number_out_of_range",
            });
        }
        digits.extend(std::iter::repeat_n('0', zero_count));
        digits
    } else {
        let scale = usize::try_from(scale).unwrap_or(usize::MAX);
        let split = digits.len().saturating_sub(scale);
        if digits[split..].bytes().any(|byte| byte != b'0') {
            return Err(KvError::InvalidValue {
                reason: "non_integral_number",
            });
        }
        if scale > digits.len() && digits.bytes().any(|byte| byte != b'0') {
            return Err(KvError::InvalidValue {
                reason: "non_integral_number",
            });
        }
        digits[..split].to_owned()
    };

    let magnitude_text = integer_digits.trim_start_matches('0');
    if magnitude_text.is_empty() {
        return Ok(0);
    }
    if magnitude_text.len() > 16
        || (magnitude_text.len() == 16 && magnitude_text > "9007199254740991")
    {
        return Err(KvError::InvalidValue {
            reason: "number_out_of_range",
        });
    }

    let magnitude = magnitude_text
        .parse::<i64>()
        .map_err(|_| KvError::InvalidValue {
            reason: "number_out_of_range",
        })?;
    Ok(if token.starts_with('-') {
        -magnitude
    } else {
        magnitude
    })
}

fn parse_saturating_exponent(text: &str) -> i64 {
    let (negative, digits) = text
        .strip_prefix('-')
        .map_or((false, text.strip_prefix('+').unwrap_or(text)), |digits| {
            (true, digits)
        });
    let magnitude = digits.bytes().fold(0_i64, |value, byte| {
        value
            .saturating_mul(10)
            .saturating_add(i64::from(byte - b'0'))
    });
    if negative { -magnitude } else { magnitude }
}
