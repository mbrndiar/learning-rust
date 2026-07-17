//! Axum adapter: translates Axum requests into [`HttpBoundary`] calls.
//!
//! This layer makes no policy decisions. Handlers pull the raw query, content
//! type, path ID, and bounded body from Axum extractors, hand them to the
//! boundary, and turn the returned [`HttpResponse`] back into an Axum response.
//! Each route registers explicit method handlers plus a fallback so unsupported
//! methods return the boundary's `405` (with `Allow`) instead of Axum's default.

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, Response, StatusCode, Uri};
use axum::routing::{MethodFilter, on};

use super::boundary::{
    ErrorReporter, HttpBoundary, HttpResponse, StderrReporter, invalid_body_response,
    method_not_allowed, route_not_found,
};
use crate::protocol::MAX_BODY_BYTES;
use crate::{ServerResult, TaskApplication, TaskService};

/// Builds the router with the default stderr error reporter.
pub fn router(service: TaskService) -> ServerResult<Router> {
    router_with_reporter(service, Arc::new(StderrReporter))
}

/// Builds the router with a caller-supplied reporter (useful in tests).
///
/// The [`HttpBoundary`] is stored as shared router state and cloned per request.
pub fn router_with_reporter(
    service: TaskService,
    reporter: Arc<dyn ErrorReporter>,
) -> ServerResult<Router> {
    let boundary = HttpBoundary::new(TaskApplication::new(service), reporter);
    Ok(Router::new()
        .route(
            "/health",
            on(MethodFilter::GET, health)
                .on(MethodFilter::HEAD, health_method_not_allowed)
                .fallback(health_method_not_allowed),
        )
        .route(
            "/tasks",
            on(MethodFilter::GET, list)
                .on(MethodFilter::HEAD, tasks_method_not_allowed)
                .on(MethodFilter::POST, create)
                .fallback(tasks_method_not_allowed),
        )
        .route(
            "/tasks/{id}",
            on(MethodFilter::GET, get)
                .on(MethodFilter::HEAD, task_method_not_allowed)
                .on(MethodFilter::PATCH, update)
                .on(MethodFilter::DELETE, delete)
                .fallback(task_method_not_allowed),
        )
        .fallback(not_found)
        .with_state(boundary))
}

async fn health(method: Method, State(boundary): State<HttpBoundary>, uri: Uri) -> Response<Body> {
    if method != Method::GET {
        return into_axum(method_not_allowed("GET"));
    }
    into_axum(boundary.health(uri.query()).await)
}

async fn list(method: Method, State(boundary): State<HttpBoundary>, uri: Uri) -> Response<Body> {
    if method != Method::GET {
        return into_axum(method_not_allowed("GET, POST"));
    }
    into_axum(boundary.list(uri.query()).await)
}

async fn create(
    State(boundary): State<HttpBoundary>,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response<Body> {
    let body = match bounded_body(body).await {
        Ok(body) => body,
        Err(response) => return into_axum(response),
    };
    into_axum(
        boundary
            .create(
                uri.query(),
                header_value(&headers, CONTENT_TYPE),
                body.as_ref(),
            )
            .await,
    )
}

async fn get(method: Method, State(boundary): State<HttpBoundary>, uri: Uri) -> Response<Body> {
    if method != Method::GET {
        return into_axum(method_not_allowed("GET, PATCH, DELETE"));
    }
    into_axum(
        boundary
            .get(raw_id(uri.path()).unwrap_or_default(), uri.query())
            .await,
    )
}

async fn update(
    State(boundary): State<HttpBoundary>,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response<Body> {
    let body = match bounded_body(body).await {
        Ok(body) => body,
        Err(response) => return into_axum(response),
    };
    into_axum(
        boundary
            .update(
                raw_id(uri.path()).unwrap_or_default(),
                uri.query(),
                header_value(&headers, CONTENT_TYPE),
                body.as_ref(),
            )
            .await,
    )
}

async fn delete(State(boundary): State<HttpBoundary>, uri: Uri) -> Response<Body> {
    into_axum(
        boundary
            .delete(raw_id(uri.path()).unwrap_or_default(), uri.query())
            .await,
    )
}

async fn health_method_not_allowed() -> Response<Body> {
    into_axum(method_not_allowed("GET"))
}

async fn tasks_method_not_allowed() -> Response<Body> {
    into_axum(method_not_allowed("GET, POST"))
}

async fn task_method_not_allowed() -> Response<Body> {
    into_axum(method_not_allowed("GET, PATCH, DELETE"))
}

async fn not_found() -> Response<Body> {
    into_axum(route_not_found())
}

// Reads the full body up to the shared cap, mapping overflow/read errors to a
// `400` so oversized or broken bodies never reach the boundary.
async fn bounded_body(body: Body) -> Result<axum::body::Bytes, HttpResponse> {
    to_bytes(body, MAX_BODY_BYTES)
        .await
        .map_err(|_| invalid_body_response())
}

// `raw_id` splits `/tasks/{id}` from the path; the boundary re-validates it, so
// a missing segment safely becomes an empty string that fails validation.
fn raw_id(path: &str) -> Option<&str> {
    let id = path.strip_prefix("/tasks/")?;
    (!id.contains('/')).then_some(id)
}

fn header_value(headers: &HeaderMap, name: axum::http::HeaderName) -> Option<&str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

// Converts the neutral boundary response into an Axum response. The boundary
// only produces valid statuses and header names, so failures are unreachable.
fn into_axum(response: HttpResponse) -> Response<Body> {
    let status =
        StatusCode::from_u16(response.status).expect("boundary only emits valid HTTP statuses");
    let mut output = Response::new(Body::from(response.body));
    *output.status_mut() = status;
    for (name, value) in response.headers {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(name), HeaderValue::try_from(value)) {
            output.headers_mut().insert(name, value);
        }
    }
    output
}
