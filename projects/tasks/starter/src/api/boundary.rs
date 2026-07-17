use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{TaskApplication, TaskError, TaskFilter, TaskPatch};

pub const MAX_BODY_BYTES: usize = 1 << 20;
pub const JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";

pub trait ErrorReporter: Send + Sync {
    fn report(&self, error: &TaskError);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WireFormatError;

#[derive(Clone, Copy, Debug, Default)]
pub struct StderrReporter;

impl ErrorReporter for StderrReporter {
    fn report(&self, error: &TaskError) {
        eprintln!("tasks-api: incomplete request: {error}");
    }
}

#[derive(Clone)]
pub struct HttpBoundary {
    service: TaskApplication,
    reporter: Arc<dyn ErrorReporter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpBoundary {
    #[must_use]
    pub fn new(service: TaskApplication, reporter: Arc<dyn ErrorReporter>) -> Self {
        Self { service, reporter }
    }

    #[must_use]
    pub const fn service(&self) -> &TaskApplication {
        &self.service
    }

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

    pub async fn health(&self, _query: Option<&str>) -> HttpResponse {
        error_response()
    }

    pub async fn list(&self, _query: Option<&str>) -> HttpResponse {
        error_response()
    }

    pub async fn get(&self, _raw_id: &str, _query: Option<&str>) -> HttpResponse {
        error_response()
    }

    pub async fn update(
        &self,
        _raw_id: &str,
        _query: Option<&str>,
        _content_type: Option<&str>,
        _body: &[u8],
    ) -> HttpResponse {
        error_response()
    }

    pub async fn delete(&self, _raw_id: &str, _query: Option<&str>) -> HttpResponse {
        error_response()
    }
}

pub fn strict_json(body: &[u8]) -> Result<Value, WireFormatError> {
    serde_json::from_slice(body).map_err(|_| WireFormatError)
}

pub fn parse_id(raw: &str) -> Result<i64, BoundaryError> {
    if raw.is_empty() || !raw.bytes().all(|value| value.is_ascii_digit()) {
        return Err(validation_id());
    }
    raw.parse::<i64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(validation_id)
}

pub fn decode_create(_content_type: Option<&str>, _body: &[u8]) -> Result<String, BoundaryError> {
    Err(incomplete_boundary())
}

pub fn decode_update(
    _content_type: Option<&str>,
    _body: &[u8],
) -> Result<TaskPatch, BoundaryError> {
    Err(incomplete_boundary())
}

pub fn validate_no_query(_query: Option<&str>) -> Result<(), BoundaryError> {
    Err(incomplete_boundary())
}

pub fn parse_list_filter(_query: Option<&str>) -> Result<TaskFilter, BoundaryError> {
    Err(incomplete_boundary())
}

#[must_use]
pub fn route_not_found() -> HttpResponse {
    error_response()
}

#[must_use]
pub fn method_not_allowed(_allow: &'static str) -> HttpResponse {
    error_response()
}

#[must_use]
pub fn invalid_body_response() -> HttpResponse {
    error_response()
}

#[must_use]
pub fn map_task_error(_error: &TaskError) -> BoundaryError {
    incomplete_boundary()
}

pub fn validate_json_content_type(content_type: Option<&str>) -> Result<(), WireFormatError> {
    content_type
        .filter(|value| value.starts_with("application/json"))
        .map(|_| ())
        .ok_or(WireFormatError)
}

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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTaskRequest {
    pub title: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}
