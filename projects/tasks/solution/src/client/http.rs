//! Reqwest client: the portable HTTP boundary the CLI and tests speak through.
//!
//! [`TaskClient`] talks only the documented wire contract, so it works against
//! either server and either backend without knowing which. It validates input
//! with the same domain rules the server uses, sends one request per call
//! (never retrying), and decodes responses strictly: status is checked first,
//! then content type, then a strict-JSON body with an exact set of fields.
//! Transport failures are classified into timeout vs. other connection errors;
//! anything the contract does not allow becomes an "unexpected response" error.

use std::time::Duration;

use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, StatusCode, Url};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::server::api::boundary::{MAX_BODY_BYTES, strict_json, validate_json_content_type};
use crate::{
    Task, TaskError, TaskFilter, TaskPatch, TaskResult, normalize_patch, normalize_title,
    validate_id,
};

/// Default request and connect timeout when none is configured.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// A client bound to one base URL, reused across calls.
#[derive(Clone, Debug)]
pub struct TaskClient {
    http: Client,
    base: Url,
    base_url: String,
    timeout: Duration,
}

// Minimal captured response: everything decoding needs, framework-independent.
#[derive(Debug)]
struct RawResponse {
    status: u16,
    content_type: Option<String>,
    body: Vec<u8>,
}

#[derive(Serialize)]
struct CreateBody<'a> {
    title: &'a str,
}

#[derive(Serialize)]
struct UpdateBody<'a> {
    // Absent fields are omitted so the server sees only the fields being patched.
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completed: Option<bool>,
}

impl TaskClient {
    /// Builds a client, normalizing `base_url` and applying `timeout` to both
    /// the connect and overall request phases; redirects are disabled so the
    /// client never silently follows a server elsewhere.
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> TaskResult<Self> {
        let (base, base_url) = parse_base_url(&base_url.into())?;
        if timeout.is_zero() {
            return Err(TaskError::client_configuration(
                "timeout",
                "timeout must be positive and finite",
            ));
        }
        let http = Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| TaskError::connection(error, false))?;
        Ok(Self {
            http,
            base,
            base_url,
            timeout,
        })
    }

    /// Borrows the underlying Reqwest client.
    #[must_use]
    pub fn http(&self) -> &Client {
        &self.http
    }

    /// The normalized base URL this client targets.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The configured request/connect timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Creates a task; validates the title locally before sending.
    pub async fn create(&self, title: &str) -> TaskResult<Task> {
        let title = normalize_title(title)?;
        let body = serde_json::to_vec(&CreateBody { title: &title })
            .map_err(|error| TaskError::internal("encode create request", error))?;
        let response = self.send(Method::POST, &["tasks"], &[], Some(body)).await?;
        decode_task_response(response, 201, &[400, 405, 422, 500])
    }

    /// Lists tasks, optionally filtered, and verifies strict ascending ID order.
    pub async fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>> {
        let query = filter
            .completed
            .map(|value| vec![("completed", if value { "true" } else { "false" })])
            .unwrap_or_default();
        let response = self.send(Method::GET, &["tasks"], &query, None).await?;
        expect_status(&response, 200, &[405, 422, 500])?;
        let value = decode_json(&response)?;
        let values = value
            .as_array()
            .ok_or_else(|| TaskError::unexpected_response("Task list response was not an array"))?;
        let mut tasks = Vec::with_capacity(values.len());
        let mut previous = 0;
        for value in values {
            let task = decode_task(value)?;
            // The contract promises strictly increasing IDs; anything else means
            // the server misbehaved, so reject rather than silently accept.
            if task.id() <= previous {
                return Err(TaskError::unexpected_response(
                    "Task list was not ordered by ascending ID",
                ));
            }
            previous = task.id();
            tasks.push(task);
        }
        Ok(tasks)
    }

    /// Fetches one task by ID.
    pub async fn get(&self, id: i64) -> TaskResult<Task> {
        validate_id(id)?;
        let id = id.to_string();
        let response = self.send(Method::GET, &["tasks", &id], &[], None).await?;
        decode_task_response(response, 200, &[404, 405, 422, 500])
    }

    /// Applies a partial update to a task.
    pub async fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task> {
        validate_id(id)?;
        let patch = normalize_patch(patch)?;
        let body = serde_json::to_vec(&UpdateBody {
            title: patch.title.as_deref(),
            completed: patch.completed,
        })
        .map_err(|error| TaskError::internal("encode update request", error))?;
        let id = id.to_string();
        let response = self
            .send(Method::PATCH, &["tasks", &id], &[], Some(body))
            .await?;
        decode_task_response(response, 200, &[400, 404, 405, 422, 500])
    }

    /// Deletes a task; a valid `204` must carry no body and no content type.
    pub async fn delete(&self, id: i64) -> TaskResult<()> {
        validate_id(id)?;
        let id = id.to_string();
        let response = self
            .send(Method::DELETE, &["tasks", &id], &[], None)
            .await?;
        expect_status(&response, 204, &[404, 405, 422, 500])?;
        if !response.body.is_empty() {
            return Err(TaskError::unexpected_response(
                "204 response body was not empty",
            ));
        }
        if response.content_type.is_some() {
            return Err(TaskError::unexpected_response(
                "204 response Content-Type was present",
            ));
        }
        Ok(())
    }

    // Builds the target URL by appending path segments and query pairs onto the
    // normalized base, so a base path prefix is preserved.
    fn build_url(&self, segments: &[&str], query: &[(&str, &str)]) -> TaskResult<Url> {
        let mut url = self.base.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| {
                TaskError::client_configuration(
                    "base-url",
                    "base URL must be an absolute HTTP or HTTPS URL",
                )
            })?;
            path.pop_if_empty();
            for segment in segments {
                path.push(segment);
            }
        }
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            pairs.clear();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        Ok(url)
    }

    // Sends exactly one request (no retry) and captures the response, enforcing
    // the body-size cap both from the advertised length and while streaming.
    async fn send(
        &self,
        method: Method,
        segments: &[&str],
        query: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> TaskResult<RawResponse> {
        let url = self.build_url(segments, query)?;
        let mut request = self.http.request(method, url);
        if let Some(body) = body {
            request = request
                .header(CONTENT_TYPE, "application/json; charset=utf-8")
                .body(body);
        }
        let mut response = request.send().await.map_err(connection_error)?;
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .map(|value| value.to_str().map(str::to_owned))
            .transpose()
            .map_err(|_| {
                TaskError::unexpected_response("response Content-Type was not application/json")
            })?;
        if response
            .content_length()
            .is_some_and(|length| length > MAX_BODY_BYTES as u64)
        {
            return Err(TaskError::unexpected_response(
                "response body exceeded 1 MiB",
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(connection_error)? {
            if bytes.len() + chunk.len() > MAX_BODY_BYTES {
                return Err(TaskError::unexpected_response(
                    "response body exceeded 1 MiB",
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(RawResponse {
            status,
            content_type,
            body: bytes,
        })
    }
}

/// Normalizes a base URL to the exact form the client stores, or reports why
/// it is unusable. Exposed so callers can validate a URL before constructing
/// a client.
pub fn normalize_base_url(raw: &str) -> TaskResult<String> {
    parse_base_url(raw).map(|(_, value)| value)
}

// Accepts only absolute http/https URLs with no credentials, query, or
// fragment, and no surrounding/internal whitespace, then trims a trailing
// slash so equivalent inputs normalize to one canonical string.
fn parse_base_url(raw: &str) -> TaskResult<(Url, String)> {
    if raw.is_empty() || raw.trim() != raw || raw.chars().any(char::is_whitespace) {
        return Err(invalid_base_url());
    }
    let mut url = Url::parse(raw).map_err(|_| invalid_base_url())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid_base_url());
    }
    let path = url.path().trim_end_matches('/').to_owned();
    url.set_path(if path.is_empty() { "/" } else { &path });
    let normalized = if path.is_empty() {
        url.as_str().trim_end_matches('/').to_owned()
    } else {
        url.as_str().to_owned()
    };
    Ok((url, normalized))
}

fn invalid_base_url() -> TaskError {
    TaskError::client_configuration("base-url", "base URL must be an absolute HTTP or HTTPS URL")
}

// Reqwest reports timeouts distinctly; preserve that so callers can tell a slow
// server from an unreachable one.
fn connection_error(error: reqwest::Error) -> TaskError {
    let timeout = error.is_timeout();
    TaskError::connection(error, timeout)
}

fn decode_task_response(
    response: RawResponse,
    success: u16,
    allowed_errors: &[u16],
) -> TaskResult<Task> {
    expect_status(&response, success, allowed_errors)?;
    let value = decode_json(&response)?;
    decode_task(&value)
}

// Status is checked before the body: only the documented success code passes,
// documented error codes decode into an API error, and anything else is an
// unexpected response.
fn expect_status(response: &RawResponse, success: u16, allowed_errors: &[u16]) -> TaskResult<()> {
    if response.status == success {
        return Ok(());
    }
    if allowed_errors.contains(&response.status) {
        return Err(decode_api_error(response)?);
    }
    Err(TaskError::unexpected_response(format!(
        "unexpected HTTP status: {}",
        response.status
    )))
}

// Requires the JSON content type and strict UTF-8 JSON body; the client is as
// strict about what it accepts as the server is about what it emits.
fn decode_json(response: &RawResponse) -> TaskResult<Value> {
    validate_json_content_type(response.content_type.as_deref()).map_err(|_| {
        TaskError::unexpected_response("response Content-Type was not application/json")
    })?;
    if std::str::from_utf8(&response.body).is_err() {
        return Err(TaskError::unexpected_response(
            "response body was not strict UTF-8 JSON",
        ));
    }
    strict_json(&response.body)
        .map_err(|_| TaskError::unexpected_response("response body was not strict UTF-8 JSON"))
}

// Requires exactly the three task fields with correct types, then rebuilds the
// value through the domain constructor so a malformed task never surfaces.
fn decode_task(value: &Value) -> TaskResult<Task> {
    let object = exact_object(value, &["completed", "id", "title"], "Task response")?;
    let id = object
        .get("id")
        .and_then(Value::as_i64)
        .ok_or_else(|| TaskError::unexpected_response("Task response values were malformed"))?;
    let title = object
        .get("title")
        .and_then(Value::as_str)
        .ok_or_else(|| TaskError::unexpected_response("Task response values were malformed"))?;
    let completed = object
        .get("completed")
        .and_then(Value::as_bool)
        .ok_or_else(|| TaskError::unexpected_response("Task response values were malformed"))?;
    Task::from_parts(id, title, completed)
        .map_err(|_| TaskError::unexpected_response("Task response values were malformed"))
}

// Decodes the error envelope and cross-checks that the machine-readable code
// matches the HTTP status, so a status/body mismatch is treated as unexpected.
fn decode_api_error(response: &RawResponse) -> TaskResult<TaskError> {
    let value = decode_json(response)?;
    let envelope = exact_object(&value, &["error"], "API error envelope")?;
    let error = envelope
        .get("error")
        .ok_or_else(|| TaskError::unexpected_response("API error value was not an object"))?;
    let error = error
        .as_object()
        .ok_or_else(|| TaskError::unexpected_response("API error value was not an object"))?;
    let keys = error.keys().map(String::as_str).collect::<Vec<_>>();
    if !keys
        .iter()
        .all(|key| matches!(*key, "code" | "message" | "details"))
        || !error.contains_key("code")
        || !error.contains_key("message")
    {
        return Err(TaskError::unexpected_response(
            "API error fields were malformed",
        ));
    }
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| TaskError::unexpected_response("API error values were malformed"))?;
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .filter(|message| !message.is_empty())
        .ok_or_else(|| TaskError::unexpected_response("API error values were malformed"))?;
    let details = match error.get("details") {
        Some(Value::Object(_)) => error.get("details").cloned(),
        Some(_) => {
            return Err(TaskError::unexpected_response(
                "API error values were malformed",
            ));
        }
        None => None,
    };
    let expected = match StatusCode::from_u16(response.status).ok() {
        Some(StatusCode::BAD_REQUEST) => "invalid_json",
        Some(StatusCode::NOT_FOUND) => "not_found",
        Some(StatusCode::METHOD_NOT_ALLOWED) => "method_not_allowed",
        Some(StatusCode::UNPROCESSABLE_ENTITY) => "validation_error",
        Some(StatusCode::INTERNAL_SERVER_ERROR) => "internal_error",
        _ => {
            return Err(TaskError::unexpected_response(format!(
                "unexpected HTTP status: {}",
                response.status
            )));
        }
    };
    if code != expected {
        return Err(TaskError::unexpected_response(format!(
            "API error code {code:?} did not match HTTP status {}",
            response.status
        )));
    }
    Ok(TaskError::api(response.status, code, message, details))
}

// Borrows an object that must contain exactly `expected` keys and no others,
// so extra or missing fields are rejected rather than ignored.
fn exact_object<'a>(
    value: &'a Value,
    expected: &[&str],
    label: &str,
) -> TaskResult<&'a Map<String, Value>> {
    let object = value
        .as_object()
        .ok_or_else(|| TaskError::unexpected_response(format!("{label} fields were malformed")))?;
    if object.len() != expected.len() || !expected.iter().all(|field| object.contains_key(*field)) {
        return Err(TaskError::unexpected_response(format!(
            "{label} fields were malformed"
        )));
    }
    Ok(object)
}
