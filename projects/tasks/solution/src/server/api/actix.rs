//! Actix adapter: the same boundary translation as the Axum adapter, in Actix
//! terms.
//!
//! Actix does not offer per-method routes as ergonomically here, so each
//! resource takes one handler that dispatches on the request method and falls
//! back to the boundary's `405`. Bodies are streamed and capped manually, then
//! the neutral [`BoundaryResponse`] is rebuilt as an Actix `HttpResponse`.
//! Keeping all decisions in the boundary is what makes this interchangeable
//! with the Axum server.

use std::sync::Arc;

use actix_web::http::header::{CONTENT_TYPE, HeaderName, HeaderValue};
use actix_web::http::{Method, StatusCode};
use actix_web::{HttpRequest, HttpResponse, Scope, web};
use futures_util::StreamExt as _;

use super::boundary::{
    ErrorReporter, HttpBoundary, HttpResponse as BoundaryResponse, StderrReporter,
    invalid_body_response, method_not_allowed, route_not_found,
};
use crate::protocol::MAX_BODY_BYTES;
use crate::{ServerResult, TaskApplication, TaskService};

/// Builds the Actix scope with the default stderr error reporter.
pub fn scope(service: TaskService) -> ServerResult<Scope> {
    Ok(scope_with_reporter(service, Arc::new(StderrReporter)))
}

/// Builds the scope with a caller-supplied reporter; the boundary is stored as
/// shared `app_data` and cloned per request.
pub(crate) fn scope_with_reporter(service: TaskService, reporter: Arc<dyn ErrorReporter>) -> Scope {
    let application = TaskApplication::new(service);
    web::scope("")
        .app_data(web::Data::new(HttpBoundary::new(application, reporter)))
        .service(web::resource("/health").route(web::route().to(health)))
        .service(web::resource("/tasks").route(web::route().to(tasks)))
        .service(web::resource("/tasks/{id}").route(web::route().to(task)))
        .default_service(web::route().to(not_found))
}

async fn health(request: HttpRequest, boundary: web::Data<HttpBoundary>) -> HttpResponse {
    let response = if request.method() == Method::GET {
        boundary.health(request.uri().query()).await
    } else {
        method_not_allowed("GET")
    };
    into_actix(response)
}

async fn tasks(
    request: HttpRequest,
    payload: web::Payload,
    boundary: web::Data<HttpBoundary>,
) -> HttpResponse {
    let response = match *request.method() {
        Method::GET => boundary.list(request.uri().query()).await,
        Method::POST => {
            let body = match bounded_body(payload).await {
                Ok(body) => body,
                Err(response) => return into_actix(response),
            };
            boundary
                .create(
                    request.uri().query(),
                    header_value(&request, CONTENT_TYPE),
                    &body,
                )
                .await
        }
        _ => method_not_allowed("GET, POST"),
    };
    into_actix(response)
}

async fn task(
    request: HttpRequest,
    payload: web::Payload,
    boundary: web::Data<HttpBoundary>,
) -> HttpResponse {
    let raw_id = raw_id(request.uri().path()).unwrap_or_default();
    let response = match *request.method() {
        Method::GET => boundary.get(raw_id, request.uri().query()).await,
        Method::PATCH => {
            let body = match bounded_body(payload).await {
                Ok(body) => body,
                Err(response) => return into_actix(response),
            };
            boundary
                .update(
                    raw_id,
                    request.uri().query(),
                    header_value(&request, CONTENT_TYPE),
                    &body,
                )
                .await
        }
        Method::DELETE => boundary.delete(raw_id, request.uri().query()).await,
        _ => method_not_allowed("GET, PATCH, DELETE"),
    };
    into_actix(response)
}

async fn not_found() -> HttpResponse {
    into_actix(route_not_found())
}

// Streams the request body chunk by chunk, enforcing the shared size cap as it
// grows so an oversized body is rejected without buffering all of it first.
async fn bounded_body(mut payload: web::Payload) -> Result<Vec<u8>, BoundaryResponse> {
    let mut body = Vec::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|_| invalid_body_response())?;
        if body.len().saturating_add(chunk.len()) > MAX_BODY_BYTES {
            return Err(invalid_body_response());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

// The boundary re-validates the ID, so a missing segment safely defaults empty.
fn raw_id(path: &str) -> Option<&str> {
    let id = path.strip_prefix("/tasks/")?;
    (!id.contains('/')).then_some(id)
}

fn header_value(request: &HttpRequest, name: HeaderName) -> Option<&str> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

// Rebuilds the neutral boundary response as an Actix response, preserving the
// empty-body distinction (204) that Actix would otherwise fill in.
fn into_actix(response: BoundaryResponse) -> HttpResponse {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut output = HttpResponse::build(status);
    for (name, value) in response.headers {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(name), HeaderValue::try_from(value)) {
            output.insert_header((name, value));
        }
    }
    if response.body.is_empty() {
        output.finish()
    } else {
        output.body(response.body)
    }
}
