//! The framework-neutral HTTP boundary you build for the API.
//!
//! Every route in either server must funnel through [`HttpBoundary`] so all
//! request policy lives in one place: content-type checks, query and path-ID
//! parsing, strict JSON decoding, status selection, and the shared error
//! envelope. Internal and storage failures must be logged through an
//! [`ErrorReporter`] and then sanitized to a generic `500`, so private detail
//! never reaches a client. The bodies here are scaffolding stubs; the shared
//! JSON and content-type helpers in [`crate::protocol`] are permissive starting
//! points you are expected to harden to the strict rules in `docs/SPEC.md`.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::JSON_CONTENT_TYPE;
use crate::{TaskApplication, TaskError, TaskFilter, TaskPatch};

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
        eprintln!("tasks-api: incomplete request: {error}");
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

    /// Handles `POST /tasks`: decode the title, then create the task.
    pub async fn create(
        &self,
        _query: Option<&str>,
        _content_type: Option<&str>,
        _body: &[u8],
    ) -> HttpResponse {
        let error = TaskError::incomplete("framework-neutral HTTP boundary");
        self.reporter.report(&error);
        error_response()
    }

    /// Handles `GET /health`.
    pub async fn health(&self, _query: Option<&str>) -> HttpResponse {
        error_response()
    }

    /// Handles `GET /tasks` with the optional `completed` filter.
    pub async fn list(&self, _query: Option<&str>) -> HttpResponse {
        error_response()
    }

    /// Handles `GET /tasks/{id}`.
    pub async fn get(&self, _raw_id: &str, _query: Option<&str>) -> HttpResponse {
        error_response()
    }

    /// Handles `PATCH /tasks/{id}`.
    pub async fn update(
        &self,
        _raw_id: &str,
        _query: Option<&str>,
        _content_type: Option<&str>,
        _body: &[u8],
    ) -> HttpResponse {
        error_response()
    }

    /// Handles `DELETE /tasks/{id}`.
    pub async fn delete(&self, _raw_id: &str, _query: Option<&str>) -> HttpResponse {
        error_response()
    }
}

/// Parses a path segment into a positive task ID (ASCII digits only), so a
/// signed or padded value is rejected before the numeric parse.
pub fn parse_id(raw: &str) -> Result<i64, BoundaryError> {
    if raw.is_empty() || !raw.bytes().all(|value| value.is_ascii_digit()) {
        return Err(validation_id());
    }
    raw.parse::<i64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(validation_id)
}

/// Decodes a `POST /tasks` body into a normalized title.
pub fn decode_create(_content_type: Option<&str>, _body: &[u8]) -> Result<String, BoundaryError> {
    Err(incomplete_boundary())
}

/// Decodes a `PATCH /tasks/{id}` body into a [`TaskPatch`].
pub fn decode_update(
    _content_type: Option<&str>,
    _body: &[u8],
) -> Result<TaskPatch, BoundaryError> {
    Err(incomplete_boundary())
}

/// Rejects any query string on routes that accept none.
pub fn validate_no_query(_query: Option<&str>) -> Result<(), BoundaryError> {
    Err(incomplete_boundary())
}

/// Parses the `GET /tasks` query into a [`TaskFilter`] (only `completed`).
pub fn parse_list_filter(_query: Option<&str>) -> Result<TaskFilter, BoundaryError> {
    Err(incomplete_boundary())
}

/// The `404` response for an unrecognized path.
#[must_use]
pub fn route_not_found() -> HttpResponse {
    error_response()
}

/// The `405` response for a known path used with the wrong method; `allow`
/// should advertise the permitted methods via an `Allow` header.
#[must_use]
pub fn method_not_allowed(_allow: &'static str) -> HttpResponse {
    error_response()
}

/// The `400` response an adapter uses when a body could not even be read.
#[must_use]
pub fn invalid_body_response() -> HttpResponse {
    error_response()
}

/// Maps a [`TaskError`] to the client-facing status and code; only validation
/// and not-found detail is safe to expose, everything else becomes a `500`.
#[must_use]
pub fn map_task_error(_error: &TaskError) -> BoundaryError {
    incomplete_boundary()
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

fn validation_id() -> BoundaryError {
    BoundaryError {
        status: 422,
        code: "validation_error",
        message: "task ID must be a positive integer".to_owned(),
        details: Some(serde_json::json!({"field": "id"})),
    }
}

// Placeholder error the stubs return until the boundary is implemented.
fn incomplete_boundary() -> BoundaryError {
    BoundaryError {
        status: 500,
        code: "internal_error",
        message: "the server could not complete the request".to_owned(),
        details: None,
    }
}

fn error_response() -> HttpResponse {
    HttpResponse {
        status: 500,
        headers: vec![("Content-Type".to_owned(), JSON_CONTENT_TYPE.to_owned())],
        body: br#"{"error":{"code":"internal_error","message":"the server could not complete the request"}}"#.to_vec(),
    }
}

/// Request DTO for `POST /tasks`; `deny_unknown_fields` rejects extra keys.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTaskRequest {
    pub title: String,
}

/// Request DTO for `PATCH /tasks/{id}`; both fields optional, extras rejected.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
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
