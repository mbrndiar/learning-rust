//! Portable HTTP/JSON rules shared by the client and server adapters.
//!
//! This module owns only wire-level mechanics that both sides must enforce:
//! response content type, body-size limits, and strict JSON parsing. It contains
//! no application service, framework, Reqwest, persistence, or CLI concerns.

use serde_json::Value;

/// Upper bound on a decoded request or response body.
pub const MAX_BODY_BYTES: usize = 1 << 20;
/// The exact `Content-Type` emitted on every JSON response.
pub const JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";

/// Marker error for strict JSON/content-type parsing failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WireFormatError;

/// Parses JSON from a request or response body.
///
/// This scaffold parses permissively; the milestone requires rejecting
/// duplicate object keys, non-finite numbers, and trailing bytes.
pub fn strict_json(body: &[u8]) -> Result<Value, WireFormatError> {
    serde_json::from_slice(body).map_err(|_| WireFormatError)
}

/// Validates a JSON `Content-Type`.
///
/// This scaffold only checks the prefix; the milestone requires the full media
/// type with a UTF-8-only charset parameter.
pub fn validate_json_content_type(content_type: Option<&str>) -> Result<(), WireFormatError> {
    content_type
        .filter(|value| value.starts_with("application/json"))
        .map(|_| ())
        .ok_or(WireFormatError)
}
