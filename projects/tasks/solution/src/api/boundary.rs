use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use serde::Serialize;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value, json};

use crate::{AsyncTaskService, TaskError, TaskFilter, TaskPatch};

pub const MAX_BODY_BYTES: usize = 1 << 20;
pub const JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";

pub trait ErrorReporter: Send + Sync {
    fn report(&self, error: &TaskError);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StderrReporter;

impl ErrorReporter for StderrReporter {
    fn report(&self, error: &TaskError) {
        eprintln!("tasks-api: internal request failure: {error}");
    }
}

#[derive(Clone)]
pub struct HttpBoundary {
    service: AsyncTaskService,
    reporter: Arc<dyn ErrorReporter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
    pub details: Option<Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WireFormatError;

impl HttpBoundary {
    #[must_use]
    pub fn new(service: AsyncTaskService, reporter: Arc<dyn ErrorReporter>) -> Self {
        Self { service, reporter }
    }

    #[must_use]
    pub const fn service(&self) -> &AsyncTaskService {
        &self.service
    }

    pub async fn health(&self, query: Option<&str>) -> HttpResponse {
        match validate_no_query(query) {
            Ok(()) => json_response(200, &HealthResponse { status: "ok" }),
            Err(error) => error_response(error),
        }
    }

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

pub fn validate_no_query(query: Option<&str>) -> Result<(), BoundaryError> {
    let parameters = parse_query(query)?;
    if let Some(key) = parameters.keys().next() {
        return Err(validation(key, format!("unknown query parameter: {key}")));
    }
    Ok(())
}

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

pub fn parse_id(raw: &str) -> Result<i64, BoundaryError> {
    if raw.is_empty() || !raw.bytes().all(|value| value.is_ascii_digit()) {
        return Err(validation("id", "task ID must be a positive integer"));
    }
    raw.parse::<i64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| validation("id", "task ID must be a positive integer"))
}

#[must_use]
pub fn route_not_found() -> HttpResponse {
    error_response(BoundaryError {
        status: 404,
        code: "not_found",
        message: "route was not found".to_owned(),
        details: None,
    })
}

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

#[must_use]
pub fn invalid_body_response() -> HttpResponse {
    error_response(invalid_json("request body must be valid JSON"))
}

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

fn decode_object(
    content_type: Option<&str>,
    body: &[u8],
) -> Result<Map<String, Value>, BoundaryError> {
    validate_request_content_type(content_type)?;
    if body.len() > MAX_BODY_BYTES || std::str::from_utf8(body).is_err() {
        return Err(invalid_json("request body must be valid JSON"));
    }
    let value = strict_json(body).map_err(|_| invalid_json("request body must be valid JSON"))?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| validation("body", "request body must be a JSON object"))
}

pub fn strict_json(body: &[u8]) -> Result<Value, WireFormatError> {
    let mut deserializer = serde_json::Deserializer::from_slice(body);
    let value = StrictValue::deserialize(&mut deserializer)
        .map(|value| value.0)
        .map_err(|_| WireFormatError)?;
    deserializer.end().map_err(|_| WireFormatError)?;
    Ok(value)
}

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

fn first_unknown(object: &Map<String, Value>, allowed: &[&str]) -> Option<String> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    object
        .keys()
        .filter(|key| !allowed.contains(key.as_str()))
        .min()
        .cloned()
}

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
