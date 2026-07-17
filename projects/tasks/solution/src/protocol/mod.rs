//! Portable HTTP/JSON rules shared by the client and server adapters.
//!
//! This module owns only wire-level mechanics that both sides must enforce:
//! response content type, body-size limits, and strict JSON parsing. It contains
//! no application service, framework, Reqwest, persistence, or CLI concerns.

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

/// Upper bound on a decoded request or response body.
pub const MAX_BODY_BYTES: usize = 1 << 20;
/// The exact `Content-Type` emitted on every JSON response.
pub const JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";

/// Marker error for strict JSON/content-type parsing failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WireFormatError;

/// Parses JSON strictly and requires the whole input to be consumed.
///
/// Rejects duplicate keys, non-finite numbers, and trailing bytes that
/// permissive parsers would accept.
pub fn strict_json(body: &[u8]) -> Result<Value, WireFormatError> {
    let mut deserializer = serde_json::Deserializer::from_slice(body);
    let value = StrictValue::deserialize(&mut deserializer)
        .map(|value| value.0)
        .map_err(|_| WireFormatError)?;
    deserializer.end().map_err(|_| WireFormatError)?;
    Ok(value)
}

/// Validates a JSON `Content-Type`, allowing only an optional UTF-8 charset.
pub fn validate_json_content_type(content_type: Option<&str>) -> Result<(), WireFormatError> {
    let raw = content_type.ok_or(WireFormatError)?;
    let mut parts = raw.split(';');
    if !parts
        .next()
        .is_some_and(|media| media.trim().eq_ignore_ascii_case("application/json"))
    {
        return Err(WireFormatError);
    }
    for parameter in parts {
        let Some((name, value)) = parameter.trim().split_once('=') else {
            return Err(WireFormatError);
        };
        if name.trim().eq_ignore_ascii_case("charset")
            && !value.trim().trim_matches('"').eq_ignore_ascii_case("utf-8")
        {
            return Err(WireFormatError);
        }
    }
    Ok(())
}

// A `serde_json::Value` wrapper whose Deserialize implementation rejects
// duplicate object keys and non-finite numbers.
struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictVisitor)
    }
}

struct StrictVisitor;

impl<'de> Visitor<'de> for StrictVisitor {
    type Value = StrictValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a strict JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .map(StrictValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate property: {key}")));
            }
            values.insert(key, object.next_value::<StrictValue>()?.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}
