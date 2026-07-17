use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, Response, StatusCode, Uri};
use axum::routing::{MethodFilter, on};

use crate::api::boundary::{
    ErrorReporter, HttpBoundary, HttpResponse, MAX_BODY_BYTES, StderrReporter,
    invalid_body_response, method_not_allowed, route_not_found,
};
use crate::{AsyncTaskService, TaskResult, TaskService};

pub fn router(service: TaskService) -> TaskResult<Router> {
    router_with_reporter(service, Arc::new(StderrReporter))
}

pub fn router_with_reporter(
    service: TaskService,
    reporter: Arc<dyn ErrorReporter>,
) -> TaskResult<Router> {
    let boundary = HttpBoundary::new(AsyncTaskService::new(service), reporter);
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
    let Some(body) = bounded_body(body).await else {
        return into_axum(invalid_body_response());
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
    let Some(body) = bounded_body(body).await else {
        return into_axum(invalid_body_response());
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

async fn bounded_body(body: Body) -> Option<axum::body::Bytes> {
    to_bytes(body, MAX_BODY_BYTES).await.ok()
}

fn raw_id(path: &str) -> Option<&str> {
    let id = path.strip_prefix("/tasks/")?;
    (!id.contains('/')).then_some(id)
}

fn header_value(headers: &HeaderMap, name: axum::http::HeaderName) -> Option<&str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

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
