//! The framework-neutral HTTP boundary: one place that owns request policy.
//!
//! Every route in either server funnels through [`HttpBoundary`]. It validates
//! content types, parses queries and path IDs, decodes JSON strictly (unknown
//! properties, duplicate keys, and non-finite numbers are rejected), calls the
//! application, and encodes the [`HttpResponse`] with the shared error envelope.
//! Adapters supply raw pieces (query string, content type, body bytes) and
//! translate the returned response; they never make status or shape decisions.
//! Internal and storage failures are logged through an [`ErrorReporter`] and
//! then sanitized to a generic `500`, so a client never sees private detail.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use serde::Serialize;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value, json};

use crate::{TaskApplication, TaskError, TaskFilter, TaskPatch};

/// Upper bound on a decoded request body; larger bodies are rejected as invalid.
pub const MAX_BODY_BYTES: usize = 1 << 20;
/// The exact `Content-Type` emitted on every JSON response.
pub const JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";

/// Sink for internal failures the boundary sanitizes before responding.
///
/// Implementations receive the full, private error for logging; the client only
/// ever sees the generic sanitized envelope.
pub trait ErrorReporter: Send + Sync {
    /// Records one internal or storage failure.
    fn report(&self, error: &TaskError);
}

/// Default reporter that writes one line to stderr.
#[derive(Clone, Copy, Debug, Default)]
pub struct StderrReporter;

impl ErrorReporter for StderrReporter {
    fn report(&self, error: &TaskError) {
        eprintln!("tasks-api: internal request failure: {error}");
    }
}

/// The shared HTTP policy: an application plus a reporter, cloned per request.
#[derive(Clone)]
pub struct HttpBoundary {
    service: TaskApplication,
    reporter: Arc<dyn ErrorReporter>,
}

/// A fully-formed HTTP response in framework-neutral terms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Wire body for `GET /health`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// The single JSON error envelope every failure serializes into.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

/// The `error` object inside an [`ErrorEnvelope`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    /// Omitted from JSON when absent, so clients see no empty `details` key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// An internal representation of one error before it becomes a response: the
/// HTTP status, machine-readable `code`, message, and optional details.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
    pub details: Option<Value>,
}

/// Marker error for strict JSON/content-type parsing failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WireFormatError;

impl HttpBoundary {
    /// Builds a boundary over an async application and an error reporter.
    #[must_use]
    pub fn new(service: TaskApplication, reporter: Arc<dyn ErrorReporter>) -> Self {
        Self { service, reporter }
    }

    /// Borrows the underlying application.
    #[must_use]
    pub const fn service(&self) -> &TaskApplication {
        &self.service
    }

    /// Handles `GET /health`; rejects any query string.
    pub async fn health(&self, query: Option<&str>) -> HttpResponse {
        match validate_no_query(query) {
            Ok(()) => json_response(200, &HealthResponse { status: "ok" }),
            Err(error) => error_response(error),
        }
    }

    /// Handles `POST /tasks`; decodes the title, then creates the task.
    pub async fn create(
        &self,
        query: Option<&str>,
        content_type: Option<&str>,
        body: &[u8],
    ) -> HttpResponse {
        let title = match validate_no_query(query).and_then(|()| decode_create(content_type, body))
        {
            Ok(title) => title,
            Err(error) => return error_response(error),
        };
        match self.service.create(title).await {
            Ok(task) => json_response(201, &task),
            Err(error) => self.service_error(error),
        }
    }

    /// Handles `GET /tasks`; parses the optional `completed` filter.
    pub async fn list(&self, query: Option<&str>) -> HttpResponse {
        let filter = match parse_list_filter(query) {
            Ok(filter) => filter,
            Err(error) => return error_response(error),
        };
        match self.service.list(filter).await {
            Ok(tasks) => json_response(200, &tasks),
            Err(error) => self.service_error(error),
        }
    }

    /// Handles `GET /tasks/{id}`.
    pub async fn get(&self, raw_id: &str, query: Option<&str>) -> HttpResponse {
        let id = match validate_no_query(query).and_then(|()| parse_id(raw_id)) {
            Ok(id) => id,
            Err(error) => return error_response(error),
        };
        match self.service.get(id).await {
            Ok(task) => json_response(200, &task),
            Err(error) => self.service_error(error),
        }
    }

    /// Handles `PATCH /tasks/{id}`; decodes the partial update, then applies it.
    pub async fn update(
        &self,
        raw_id: &str,
        query: Option<&str>,
        content_type: Option<&str>,
        body: &[u8],
    ) -> HttpResponse {
        let id = match validate_no_query(query).and_then(|()| parse_id(raw_id)) {
            Ok(id) => id,
            Err(error) => return error_response(error),
        };
        let patch = match decode_update(content_type, body) {
            Ok(patch) => patch,
            Err(error) => return error_response(error),
        };
        match self.service.update(id, patch).await {
            Ok(task) => json_response(200, &task),
            Err(error) => self.service_error(error),
        }
    }

    /// Handles `DELETE /tasks/{id}`; returns `204` with an empty body on success.
    pub async fn delete(&self, raw_id: &str, query: Option<&str>) -> HttpResponse {
        let id = match validate_no_query(query).and_then(|()| parse_id(raw_id)) {
            Ok(id) => id,
            Err(error) => return error_response(error),
        };
        match self.service.delete(id).await {
            Ok(()) => HttpResponse {
                status: 204,
                headers: Vec::new(),
                body: Vec::new(),
            },
            Err(error) => self.service_error(error),
        }
    }

    // Turns a service failure into a response. Storage/internal/incomplete
    // errors carry private context, so they are logged and then mapped to a
    // sanitized `500`; validation and not-found errors map to their own status.
    fn service_error(&self, error: TaskError) -> HttpResponse {
        if matches!(
            error,
            TaskError::Storage { .. } | TaskError::Internal { .. } | TaskError::Incomplete { .. }
        ) {
            self.reporter.report(&error);
        }
        error_response(map_task_error(&error))
    }
}

/// Decodes a `POST /tasks` body into a normalized title.
///
/// Rejects unknown properties, wrong types, and missing `title` before running
/// the same domain normalization the service uses.
pub fn decode_create(content_type: Option<&str>, body: &[u8]) -> Result<String, BoundaryError> {
    let object = decode_object(content_type, body)?;
    if let Some(field) = first_unknown(&object, &["title"]) {
        return Err(validation(&field, format!("unknown property: {field}")));
    }
    let title = object
        .get("title")
        .ok_or_else(|| validation("title", "missing property: title"))?
        .as_str()
        .ok_or_else(|| validation("title", "title must be a string"))?;
    crate::normalize_title(title).map_err(|error| map_task_error(&error))
}

/// Decodes a `PATCH /tasks/{id}` body into a [`TaskPatch`].
///
/// Both fields are optional; each present field is type-checked, then the patch
/// runs through the same domain normalization the service uses.
pub fn decode_update(content_type: Option<&str>, body: &[u8]) -> Result<TaskPatch, BoundaryError> {
    let object = decode_object(content_type, body)?;
    if let Some(field) = first_unknown(&object, &["completed", "title"]) {
        return Err(validation(&field, format!("unknown property: {field}")));
    }
    let title = match object.get("title") {
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => return Err(validation("title", "title must be a string")),
        None => None,
    };
    let completed = match object.get("completed") {
        Some(Value::Bool(value)) => Some(*value),
        Some(_) => return Err(validation("completed", "completed must be a Boolean")),
        None => None,
    };
    crate::normalize_patch(TaskPatch { title, completed }).map_err(|error| map_task_error(&error))
}

/// Rejects any query string on routes that accept none.
pub fn validate_no_query(query: Option<&str>) -> Result<(), BoundaryError> {
    let parameters = parse_query(query)?;
    if let Some(key) = parameters.keys().next() {
        return Err(validation(key, format!("unknown query parameter: {key}")));
    }
    Ok(())
}

/// Parses the `GET /tasks` query into a [`TaskFilter`].
///
/// Only a single `completed=true|false` is accepted; anything else is a
/// validation error so filters stay unambiguous.
pub fn parse_list_filter(query: Option<&str>) -> Result<TaskFilter, BoundaryError> {
    let parameters = parse_query(query)?;
    if let Some(key) = parameters.keys().find(|key| key.as_str() != "completed") {
        return Err(validation(key, format!("unknown query parameter: {key}")));
    }
    let Some(values) = parameters.get("completed") else {
        return Ok(TaskFilter::default());
    };
    if values.len() != 1 {
        return Err(validation(
            "completed",
            "completed filter must be true or false",
        ));
    }
    let completed = match values[0].as_str() {
        "true" => true,
        "false" => false,
        _ => {
            return Err(validation(
                "completed",
                "completed filter must be true or false",
            ));
        }
    };
    Ok(TaskFilter {
        completed: Some(completed),
    })
}

/// Parses a path segment into a positive task ID.
///
/// Only ASCII digits are accepted, so leading `+`, whitespace, or signs are
/// rejected before the numeric parse.
pub fn parse_id(raw: &str) -> Result<i64, BoundaryError> {
    if raw.is_empty() || !raw.bytes().all(|value| value.is_ascii_digit()) {
        return Err(validation("id", "task ID must be a positive integer"));
    }
    raw.parse::<i64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| validation("id", "task ID must be a positive integer"))
}

/// The `404` response for an unrecognized path.
#[must_use]
pub fn route_not_found() -> HttpResponse {
    error_response(BoundaryError {
        status: 404,
        code: "not_found",
        message: "route was not found".to_owned(),
        details: None,
    })
}

/// The `405` response for a known path used with the wrong method; `allow`
/// becomes the `Allow` header advertising the permitted methods.
#[must_use]
pub fn method_not_allowed(allow: &'static str) -> HttpResponse {
    let mut response = error_response(BoundaryError {
        status: 405,
        code: "method_not_allowed",
        message: "method is not allowed for this path".to_owned(),
        details: None,
    });
    response
        .headers
        .push(("Allow".to_owned(), allow.to_owned()));
    response
}

/// The `400` response an adapter uses when a body could not even be read.
#[must_use]
pub fn invalid_body_response() -> HttpResponse {
    error_response(invalid_json("request body must be valid JSON"))
}

/// Maps a [`TaskError`] to the client-facing status and code.
///
/// Only validation and not-found detail is safe to expose; every other variant
/// collapses to a generic `500` so private context never leaks.
#[must_use]
pub fn map_task_error(error: &TaskError) -> BoundaryError {
    match error {
        TaskError::Validation { field, message } => validation(field, message),
        TaskError::NotFound { id } => BoundaryError {
            status: 404,
            code: "not_found",
            message: format!("task {id} was not found"),
            details: None,
        },
        _ => BoundaryError {
            status: 500,
            code: "internal_error",
            message: "the server could not complete the request".to_owned(),
            details: None,
        },
    }
}

// Shared body decode: enforce content type, cap size, require UTF-8, parse
// strictly, and require a top-level object before field checks run.
fn decode_object(
    content_type: Option<&str>,
    body: &[u8],
) -> Result<Map<String, Value>, BoundaryError> {
    validate_request_content_type(content_type)?;
    if body.len() > MAX_BODY_BYTES {
        return Err(invalid_json("request body must be valid JSON"));
    }
    if std::str::from_utf8(body).is_err() {
        return Err(invalid_json("request body must be valid JSON"));
    }
    let value = strict_json(body).map_err(|_| invalid_json("request body must be valid JSON"))?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| validation("body", "request body must be a JSON object"))
}

/// Parses JSON strictly and requires the whole input to be consumed.
///
/// Rejects duplicate keys, non-finite numbers, and trailing bytes that
/// permissive parsers would accept. See `StrictValue`.
pub fn strict_json(body: &[u8]) -> Result<Value, WireFormatError> {
    let mut deserializer = serde_json::Deserializer::from_slice(body);
    let value = StrictValue::deserialize(&mut deserializer)
        .map(|value| value.0)
        .map_err(|_| WireFormatError)?;
    // `end` rejects trailing content after one JSON value.
    deserializer.end().map_err(|_| WireFormatError)?;
    Ok(value)
}

/// Validates a response `Content-Type` as `application/json` (UTF-8 only).
///
/// Used by the client to check what a server sent; the request-side check
/// lives in `validate_request_content_type`.
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

// Request-side content-type check: `application/json` with an optional
// UTF-8 charset. Mirrors the response-side check but returns a `400` envelope.
fn validate_request_content_type(content_type: Option<&str>) -> Result<(), BoundaryError> {
    let raw = content_type
        .ok_or_else(|| invalid_json("request Content-Type must be application/json"))?;
    let mut parts = raw.split(';');
    if !parts
        .next()
        .is_some_and(|media| media.trim().eq_ignore_ascii_case("application/json"))
    {
        return Err(invalid_json(
            "request Content-Type must be application/json",
        ));
    }
    for parameter in parts {
        let Some((name, value)) = parameter.trim().split_once('=') else {
            return Err(invalid_json(
                "request Content-Type must be application/json",
            ));
        };
        if name.trim().eq_ignore_ascii_case("charset")
            && !value.trim().trim_matches('"').eq_ignore_ascii_case("utf-8")
        {
            return Err(invalid_json("request JSON charset must be UTF-8"));
        }
    }
    Ok(())
}

// Parses `key=value&...` into multi-valued parameters. Keys and values are
// percent-decoded; `+` maps to space. Ordering is deterministic via BTreeMap.
fn parse_query(query: Option<&str>) -> Result<BTreeMap<String, Vec<String>>, BoundaryError> {
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let Some(query) = query.filter(|query| !query.is_empty()) else {
        return Ok(result);
    };
    for parameter in query.split('&') {
        let (key, value) = parameter.split_once('=').unwrap_or((parameter, ""));
        let key = percent_decode(key)
            .ok_or_else(|| validation("query", "query string must be valid URL encoding"))?;
        let value = percent_decode(value)
            .ok_or_else(|| validation(&key, format!("invalid query parameter: {key}")))?;
        result.entry(key).or_default().push(value);
    }
    Ok(result)
}

// Manual percent-decoder: `%XX` -> byte, `+` -> space. Returns None on a
// malformed escape or non-UTF-8 result so callers can report invalid encoding.
fn percent_decode(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                decoded.push((hex(bytes[index + 1])? << 4) | hex(bytes[index + 2])?);
                index += 3;
            }
            b'%' => return None,
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            value => {
                decoded.push(value);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).ok()
}

const fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

// Returns the lexicographically smallest property not in `allowed`, so error
// messages about unknown fields are deterministic regardless of input order.
fn first_unknown(object: &Map<String, Value>, allowed: &[&str]) -> Option<String> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    object
        .keys()
        .filter(|key| !allowed.contains(key.as_str()))
        .min()
        .cloned()
}

// Serializes a trusted value into a JSON response. The payloads here are
// server-owned, so serialization failure would be a programmer error.
fn json_response(status: u16, value: &impl Serialize) -> HttpResponse {
    HttpResponse {
        status,
        headers: vec![("Content-Type".to_owned(), JSON_CONTENT_TYPE.to_owned())],
        body: serde_json::to_vec(value).expect("serializing trusted HTTP response cannot fail"),
    }
}

fn error_response(error: BoundaryError) -> HttpResponse {
    json_response(
        error.status,
        &ErrorEnvelope {
            error: ErrorBody {
                code: error.code.to_owned(),
                message: error.message,
                details: error.details,
            },
        },
    )
}

fn validation(field: &str, message: impl Into<String>) -> BoundaryError {
    BoundaryError {
        status: 422,
        code: "validation_error",
        message: message.into(),
        details: Some(json!({ "field": field })),
    }
}

fn invalid_json(message: impl Into<String>) -> BoundaryError {
    BoundaryError {
        status: 400,
        code: "invalid_json",
        message: message.into(),
        details: None,
    }
}

// A `serde_json::Value` wrapper whose Deserialize enforces stricter rules than
// the default: no duplicate object keys and no non-finite numbers.
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
        // `from_f64` returns None for NaN/Infinity, which JSON cannot represent.
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
            // A duplicate key is ambiguous, so reject rather than silently
            // keeping first or last.
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate property: {key}")));
            }
            values.insert(key, object.next_value::<StrictValue>()?.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}
