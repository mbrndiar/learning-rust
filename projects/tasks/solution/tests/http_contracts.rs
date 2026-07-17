use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderValue, Response, StatusCode};
use axum::routing::get;
use serde_json::{Value, json};
use tasks_solution::api::boundary::{
    ErrorReporter, HttpBoundary, JSON_CONTENT_TYPE, MAX_BODY_BYTES,
};
use tasks_solution::client::TaskClient;
use tasks_solution::server::{BackendKind, ServerConfig, ServerKind};
use tasks_solution::{
    AsyncTaskService, Task, TaskError, TaskFilter, TaskPatch, TaskRepository, TaskResult,
    TaskService,
};
use tokio::sync::oneshot;

struct MemoryRepository {
    state: Mutex<(i64, Vec<Task>)>,
}

impl MemoryRepository {
    fn new() -> Self {
        Self {
            state: Mutex::new((0, Vec::new())),
        }
    }
}

impl TaskRepository for MemoryRepository {
    fn create(&self, title: &str) -> TaskResult<Task> {
        let mut state = self.state.lock().expect("lock memory repository");
        state.0 += 1;
        let task = Task::from_parts(state.0, title, false)?;
        state.1.push(task.clone());
        Ok(task)
    }

    fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>> {
        let state = self.state.lock().expect("lock memory repository");
        Ok(state
            .1
            .iter()
            .filter(|task| {
                filter
                    .completed
                    .is_none_or(|value| task.completed() == value)
            })
            .cloned()
            .collect())
    }

    fn get(&self, id: i64) -> TaskResult<Task> {
        self.state
            .lock()
            .expect("lock memory repository")
            .1
            .iter()
            .find(|task| task.id() == id)
            .cloned()
            .ok_or_else(|| TaskError::not_found(id))
    }

    fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task> {
        let mut state = self.state.lock().expect("lock memory repository");
        let task = state
            .1
            .iter_mut()
            .find(|task| task.id() == id)
            .ok_or_else(|| TaskError::not_found(id))?;
        *task = Task::from_parts(
            id,
            patch.title.as_deref().unwrap_or(task.title()),
            patch.completed.unwrap_or(task.completed()),
        )?;
        Ok(task.clone())
    }

    fn delete(&self, id: i64) -> TaskResult<()> {
        let mut state = self.state.lock().expect("lock memory repository");
        let index = state
            .1
            .iter()
            .position(|task| task.id() == id)
            .ok_or_else(|| TaskError::not_found(id))?;
        state.1.remove(index);
        Ok(())
    }
}

#[derive(Default)]
struct RecordingReporter {
    messages: Mutex<Vec<String>>,
}

impl ErrorReporter for RecordingReporter {
    fn report(&self, error: &TaskError) {
        self.messages
            .lock()
            .expect("lock reports")
            .push(error.to_string());
    }
}

struct FailingRepository;

impl TaskRepository for FailingRepository {
    fn create(&self, _: &str) -> TaskResult<Task> {
        self.fail()
    }
    fn list(&self, _: TaskFilter) -> TaskResult<Vec<Task>> {
        self.fail()
    }
    fn get(&self, _: i64) -> TaskResult<Task> {
        self.fail()
    }
    fn update(&self, _: i64, _: TaskPatch) -> TaskResult<Task> {
        self.fail()
    }
    fn delete(&self, _: i64) -> TaskResult<()> {
        self.fail()
    }
}

impl FailingRepository {
    fn fail<T>(&self) -> TaskResult<T> {
        Err(TaskError::storage(
            "test",
            io::Error::other("private storage detail"),
        ))
    }
}

struct PanicRepository;

impl TaskRepository for PanicRepository {
    fn create(&self, _: &str) -> TaskResult<Task> {
        panic!("private panic")
    }
    fn list(&self, _: TaskFilter) -> TaskResult<Vec<Task>> {
        panic!("private panic")
    }
    fn get(&self, _: i64) -> TaskResult<Task> {
        panic!("private panic")
    }
    fn update(&self, _: i64, _: TaskPatch) -> TaskResult<Task> {
        panic!("private panic")
    }
    fn delete(&self, _: i64) -> TaskResult<()> {
        panic!("private panic")
    }
}

fn boundary(repository: Arc<dyn TaskRepository>) -> HttpBoundary {
    HttpBoundary::new(
        AsyncTaskService::new(TaskService::new(repository)),
        Arc::new(RecordingReporter::default()),
    )
}

fn error_parts(
    response: &tasks_solution::api::boundary::HttpResponse,
) -> (String, String, Option<String>) {
    let value: Value = serde_json::from_slice(&response.body).expect("decode error response");
    let error = &value["error"];
    (
        error["code"].as_str().expect("error code").to_owned(),
        error["message"].as_str().expect("error message").to_owned(),
        error
            .get("details")
            .and_then(|details| details.get("field"))
            .and_then(Value::as_str)
            .map(str::to_owned),
    )
}

type InvalidBodyCase<'a> = (
    &'a str,
    Option<&'a str>,
    Vec<u8>,
    u16,
    &'a str,
    &'a str,
    Option<&'a str>,
);

#[tokio::test(flavor = "multi_thread")]
async fn strict_framework_neutral_request_boundary() {
    let boundary = boundary(Arc::new(MemoryRepository::new()));
    let mut oversized = vec![b' '; MAX_BODY_BYTES + 1];
    oversized[0] = b'{';
    let cases: Vec<InvalidBodyCase<'_>> = vec![
        (
            "missing type",
            None,
            b"{}".to_vec(),
            400,
            "invalid_json",
            "request Content-Type must be application/json",
            None,
        ),
        (
            "wrong type",
            Some("text/plain"),
            b"{}".to_vec(),
            400,
            "invalid_json",
            "request Content-Type must be application/json",
            None,
        ),
        (
            "wrong charset",
            Some("application/json; charset=iso-8859-1"),
            b"{}".to_vec(),
            400,
            "invalid_json",
            "request JSON charset must be UTF-8",
            None,
        ),
        (
            "invalid UTF-8",
            Some("application/json"),
            vec![0xff],
            400,
            "invalid_json",
            "request body must be valid JSON",
            None,
        ),
        (
            "malformed",
            Some("application/json"),
            b"{".to_vec(),
            400,
            "invalid_json",
            "request body must be valid JSON",
            None,
        ),
        (
            "duplicate",
            Some("application/json"),
            br#"{"title":"a","title":"b"}"#.to_vec(),
            400,
            "invalid_json",
            "request body must be valid JSON",
            None,
        ),
        (
            "nested duplicate",
            Some("application/json"),
            br#"{"title":"a","x":{"a":1,"a":2}}"#.to_vec(),
            400,
            "invalid_json",
            "request body must be valid JSON",
            None,
        ),
        (
            "trailing",
            Some("application/json"),
            br#"{"title":"a"} {}"#.to_vec(),
            400,
            "invalid_json",
            "request body must be valid JSON",
            None,
        ),
        (
            "oversized",
            Some("application/json"),
            oversized,
            400,
            "invalid_json",
            "request body must be valid JSON",
            None,
        ),
        (
            "shape",
            Some("application/json"),
            b"[]".to_vec(),
            422,
            "validation_error",
            "request body must be a JSON object",
            Some("body"),
        ),
        (
            "missing",
            Some("application/json"),
            b"{}".to_vec(),
            422,
            "validation_error",
            "missing property: title",
            Some("title"),
        ),
        (
            "unknown",
            Some("application/json"),
            br#"{"title":"x","done":false}"#.to_vec(),
            422,
            "validation_error",
            "unknown property: done",
            Some("done"),
        ),
        (
            "null",
            Some("application/json"),
            br#"{"title":null}"#.to_vec(),
            422,
            "validation_error",
            "title must be a string",
            Some("title"),
        ),
        (
            "wrong type",
            Some("application/json"),
            br#"{"title":7}"#.to_vec(),
            422,
            "validation_error",
            "title must be a string",
            Some("title"),
        ),
        (
            "empty",
            Some("application/json"),
            br#"{"title":" "}"#.to_vec(),
            422,
            "validation_error",
            "title must contain between 1 and 120 characters",
            Some("title"),
        ),
    ];
    for (name, content_type, body, status, code, message, field) in cases {
        let response = boundary.create(None, content_type, &body).await;
        assert_eq!(response.status, status, "{name}");
        assert_eq!(
            error_parts(&response),
            (
                code.to_owned(),
                message.to_owned(),
                field.map(str::to_owned)
            ),
            "{name}"
        );
        assert_eq!(
            response.headers,
            vec![("Content-Type".to_owned(), JSON_CONTENT_TYPE.to_owned())],
            "{name}"
        );
    }

    let response = boundary
        .create(
            None,
            Some("application/json"),
            br#"{"title":"  Learn REST  "}"#,
        )
        .await;
    assert_eq!(response.status, 201);
    assert_eq!(
        serde_json::from_slice::<Value>(&response.body).expect("decode created task"),
        json!({"id": 1, "title": "Learn REST", "completed": false})
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn strict_patch_query_and_id_boundary() {
    let boundary = boundary(Arc::new(MemoryRepository::new()));
    let created = boundary
        .create(None, Some("application/json"), br#"{"title":"task"}"#)
        .await;
    assert_eq!(created.status, 201);

    for (body, message, field) in [
        (
            br#"{}"#.as_slice(),
            "update must include title or completed",
            "update",
        ),
        (
            br#"{"completed":null}"#.as_slice(),
            "completed must be a Boolean",
            "completed",
        ),
        (
            br#"{"completed":0}"#.as_slice(),
            "completed must be a Boolean",
            "completed",
        ),
        (
            br#"{"title":null}"#.as_slice(),
            "title must be a string",
            "title",
        ),
    ] {
        let response = boundary
            .update("1", None, Some("application/json"), body)
            .await;
        assert_eq!(response.status, 422);
        assert_eq!(error_parts(&response).1, message);
        assert_eq!(error_parts(&response).2.as_deref(), Some(field));
    }

    for (query, message, field) in [
        (
            Some("completed=True"),
            "completed filter must be true or false",
            "completed",
        ),
        (
            Some("completed=true&completed=false"),
            "completed filter must be true or false",
            "completed",
        ),
        (
            Some("other=true"),
            "unknown query parameter: other",
            "other",
        ),
    ] {
        let response = boundary.list(query).await;
        assert_eq!(response.status, 422);
        assert_eq!(error_parts(&response).1, message);
        assert_eq!(error_parts(&response).2.as_deref(), Some(field));
    }
    for id in ["0", "+1", "١", "1.0", "", "9223372036854775808"] {
        let response = boundary.get(id, None).await;
        assert_eq!(response.status, 422, "{id:?}");
        assert_eq!(
            error_parts(&response).1,
            "task ID must be a positive integer"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn blocking_facade_and_internal_reporting_are_sanitized() {
    let panic_service = AsyncTaskService::new(TaskService::new(Arc::new(PanicRepository)));
    let error = panic_service
        .list(TaskFilter::default())
        .await
        .expect_err("panic must become an internal error");
    assert!(matches!(error, TaskError::Internal { .. }));

    let reporter = Arc::new(RecordingReporter::default());
    let boundary = HttpBoundary::new(
        AsyncTaskService::new(TaskService::new(Arc::new(FailingRepository))),
        reporter.clone(),
    );
    let response = boundary.list(None).await;
    assert_eq!(response.status, 500);
    let body = String::from_utf8(response.body).expect("UTF-8 error body");
    assert!(!body.contains("private"));
    assert!(body.contains("the server could not complete the request"));
    assert!(
        reporter
            .messages
            .lock()
            .expect("lock reports")
            .iter()
            .any(|message| message.contains("private storage detail"))
    );
}

struct LiveServer {
    base_url: String,
    shutdown: Option<oneshot::Sender<()>>,
    join: tokio::task::JoinHandle<TaskResult<()>>,
}

impl LiveServer {
    async fn stop(mut self) {
        self.shutdown.take().expect("shutdown sender").send(()).ok();
        tokio::time::timeout(Duration::from_secs(2), self.join)
            .await
            .expect("server shutdown timeout")
            .expect("join server")
            .expect("serve server");
    }
}

async fn start_config(config: ServerConfig) -> LiveServer {
    let server = tasks_solution::server::bind(config)
        .await
        .expect("bind task server");
    let base_url = format!("http://{}", server.local_addr());
    let (shutdown, receiver) = oneshot::channel();
    let join = tokio::spawn(server.serve(async {
        receiver.await.ok();
    }));
    LiveServer {
        base_url,
        shutdown: Some(shutdown),
        join,
    }
}

fn config(backend: BackendKind, data: std::path::PathBuf) -> ServerConfig {
    ServerConfig {
        server: ServerKind::Axum,
        backend,
        data,
        host: "127.0.0.1".to_owned(),
        port: 0,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn axum_crud_works_with_both_repositories_and_restart() {
    for (backend, filename) in [
        (BackendKind::Sqlite, "tasks.db"),
        (BackendKind::Markdown, "tasks.md"),
    ] {
        let directory = tempfile::tempdir().expect("temporary storage");
        let data = directory.path().join(filename);
        let live = start_config(config(backend, data.clone())).await;
        let client = TaskClient::new(&live.base_url, Duration::from_secs(2)).expect("client");
        let created = client.create("  Learn Axum  ").await.expect("create");
        assert_eq!(created.id(), 1);
        let updated = client
            .update(
                1,
                TaskPatch {
                    title: None,
                    completed: Some(true),
                },
            )
            .await
            .expect("update");
        assert!(updated.completed());
        assert_eq!(
            client
                .list(TaskFilter {
                    completed: Some(true)
                })
                .await
                .expect("list"),
            vec![updated]
        );
        live.stop().await;

        let restarted = start_config(config(backend, data)).await;
        let client =
            TaskClient::new(&restarted.base_url, Duration::from_secs(2)).expect("restart client");
        assert_eq!(client.get(1).await.expect("persisted task").id(), 1);
        client.delete(1).await.expect("delete");
        assert_eq!(client.create("next").await.expect("next task").id(), 2);
        restarted.stop().await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn axum_route_method_and_body_policy_is_exact() {
    let directory = tempfile::tempdir().expect("temporary storage");
    let live = start_config(config(
        BackendKind::Sqlite,
        directory.path().join("tasks.db"),
    ))
    .await;
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP client");
    for (method, path, allow) in [
        (reqwest::Method::POST, "/health", "GET"),
        (reqwest::Method::HEAD, "/health", "GET"),
        (reqwest::Method::PUT, "/tasks", "GET, POST"),
        (reqwest::Method::OPTIONS, "/tasks/1", "GET, PATCH, DELETE"),
    ] {
        let is_head = method == reqwest::Method::HEAD;
        let response = http
            .request(method, format!("{}{path}", live.base_url))
            .send()
            .await
            .expect("method response");
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers()["allow"], allow);
        assert_eq!(response.headers()["content-type"], JSON_CONTENT_TYPE);
        if !is_head {
            let value: Value = response.json().await.expect("method JSON");
            assert_eq!(value["error"]["code"], "method_not_allowed");
        }
    }
    for path in ["/missing", "/tasks/", "/tasks//", "/tasks/1/extra", "/docs"] {
        let response = http
            .get(format!("{}{path}", live.base_url))
            .send()
            .await
            .expect("route response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        let value: Value = response.json().await.expect("route JSON");
        assert_eq!(value["error"]["message"], "route was not found");
    }
    let address = live
        .base_url
        .strip_prefix("http://")
        .expect("loopback base URL")
        .to_owned();
    for path in ["/tasks/../tasks", "/tasks//"] {
        let address = address.clone();
        let raw = tokio::task::spawn_blocking(move || raw_http_get(&address, path))
            .await
            .expect("join raw request")
            .expect("raw request");
        assert!(raw.starts_with("HTTP/1.1 404"), "{path}: {raw}");
        assert!(
            raw.contains(r#""message":"route was not found""#),
            "{path}: {raw}"
        );
    }
    let delete = http
        .delete(format!("{}/tasks/999", live.base_url))
        .send()
        .await
        .expect("delete response");
    assert_eq!(delete.status(), StatusCode::NOT_FOUND);
    live.stop().await;
}

fn raw_http_get(address: &str, path: &str) -> io::Result<String> {
    use std::io::{Read as _, Write as _};

    let mut stream = std::net::TcpStream::connect(address)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

async fn response_server(
    status: StatusCode,
    content_type: Option<&str>,
    body: Vec<u8>,
    delay: Option<Duration>,
    calls: Arc<AtomicUsize>,
) -> LiveServer {
    let content_type = content_type.map(str::to_owned);
    let router = Router::new().fallback(get(move || {
        let content_type = content_type.clone();
        let body = body.clone();
        let calls = calls.clone();
        async move {
            calls.fetch_add(1, Ordering::SeqCst);
            if let Some(delay) = delay {
                tokio::time::sleep(delay).await;
            }
            let mut response = Response::new(Body::from(body));
            *response.status_mut() = status;
            if let Some(content_type) = content_type {
                response.headers_mut().insert(
                    "content-type",
                    HeaderValue::from_str(&content_type).expect("content type"),
                );
            }
            response
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("response listener");
    let base_url = format!(
        "http://{}",
        listener.local_addr().expect("response address")
    );
    let (shutdown, receiver) = oneshot::channel();
    let join = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                receiver.await.ok();
            })
            .await
            .map_err(|error| TaskError::lifecycle("test response server", error))
    });
    LiveServer {
        base_url,
        shutdown: Some(shutdown),
        join,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn reqwest_client_rejects_malformed_responses_and_does_not_retry() {
    let cases = [
        (200, None, br#"{}"#.as_slice()),
        (200, Some("text/plain"), br#"{}"#.as_slice()),
        (
            200,
            Some("application/json"),
            br#"{"id":7,"id":8,"title":"x","completed":false}"#.as_slice(),
        ),
        (
            200,
            Some("application/json"),
            br#"{"id":7,"title":" padded ","completed":false}"#.as_slice(),
        ),
        (
            201,
            Some("application/json"),
            br#"{"id":7,"title":"x","completed":false}"#.as_slice(),
        ),
    ];
    for (status, content_type, body) in cases {
        let calls = Arc::new(AtomicUsize::new(0));
        let live = response_server(
            StatusCode::from_u16(status).expect("status"),
            content_type,
            body.to_vec(),
            None,
            calls,
        )
        .await;
        let client = TaskClient::new(&live.base_url, Duration::from_secs(1)).expect("client");
        let error = client.get(7).await.expect_err("malformed response");
        assert!(error.unexpected_response_message().is_some());
        live.stop().await;
    }

    let calls = Arc::new(AtomicUsize::new(0));
    let live = response_server(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some("application/json"),
        br#"{"error":{"code":"internal_error","message":"failure"}}"#.to_vec(),
        None,
        calls.clone(),
    )
    .await;
    let client = TaskClient::new(&live.base_url, Duration::from_secs(1)).expect("client");
    assert!(
        client
            .list(TaskFilter::default())
            .await
            .expect_err("API error")
            .api_details()
            .is_some()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    live.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn reqwest_client_bounds_responses_connection_and_timeout() {
    let calls = Arc::new(AtomicUsize::new(0));
    let live = response_server(
        StatusCode::OK,
        Some("application/json"),
        vec![b' '; MAX_BODY_BYTES + 1],
        None,
        calls,
    )
    .await;
    let client = TaskClient::new(&live.base_url, Duration::from_secs(1)).expect("client");
    assert!(
        client
            .list(TaskFilter::default())
            .await
            .expect_err("oversized response")
            .unexpected_response_message()
            .is_some()
    );
    live.stop().await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("unavailable listener");
    let address = listener.local_addr().expect("unavailable address");
    drop(listener);
    let client =
        TaskClient::new(format!("http://{address}"), Duration::from_millis(100)).expect("client");
    assert!(
        client
            .list(TaskFilter::default())
            .await
            .expect_err("connection")
            .is_connection()
    );

    let calls = Arc::new(AtomicUsize::new(0));
    let live = response_server(
        StatusCode::OK,
        Some("application/json"),
        b"[]".to_vec(),
        Some(Duration::from_millis(200)),
        calls,
    )
    .await;
    let client = TaskClient::new(&live.base_url, Duration::from_millis(20)).expect("client");
    let error = client
        .list(TaskFilter::default())
        .await
        .expect_err("timeout");
    assert!(error.is_timeout());
    live.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn cli_factory_output_and_exit_categories_are_stable() {
    let called = Arc::new(AtomicBool::new(false));
    let marker = called.clone();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = tasks_solution::cli::run_from_with_factory(
        ["tasks", "show", "0"],
        move |_, _| {
            marker.store(true, Ordering::SeqCst);
            panic!("factory must not be called")
        },
        &mut stdout,
        &mut stderr,
    )
    .await;
    assert_eq!(exit, 2);
    assert!(!called.load(Ordering::SeqCst));
    assert!(stdout.is_empty());
    assert_eq!(
        String::from_utf8(stderr)
            .expect("usage stderr")
            .lines()
            .count(),
        1
    );

    let directory = tempfile::tempdir().expect("temporary storage");
    let live = start_config(config(
        BackendKind::Sqlite,
        directory.path().join("tasks.db"),
    ))
    .await;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = tasks_solution::cli::run_from_with_factory(
        ["tasks", "--base-url", &live.base_url, "add", "CLI task"],
        TaskClient::new,
        &mut stdout,
        &mut stderr,
    )
    .await;
    assert_eq!(exit, 0);
    assert_eq!(
        String::from_utf8(stdout).expect("CLI stdout"),
        "{\"id\":1,\"title\":\"CLI task\",\"completed\":false}\n"
    );
    assert!(stderr.is_empty());

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = tasks_solution::cli::run_from_with_factory(
        ["tasks", "--base-url", &live.base_url, "show", "99"],
        TaskClient::new,
        &mut stdout,
        &mut stderr,
    )
    .await;
    assert_eq!(exit, 3);
    assert!(stdout.is_empty());
    assert_eq!(
        String::from_utf8(stderr).expect("API stderr"),
        "api: 404 not_found: task 99 was not found\n"
    );
    live.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn server_lifecycle_is_repeatable_and_concurrent() {
    for iteration in 0..3 {
        let directory = tempfile::tempdir().expect("temporary storage");
        let live = start_config(config(
            BackendKind::Sqlite,
            directory.path().join(format!("tasks-{iteration}.db")),
        ))
        .await;
        let client =
            TaskClient::new(&live.base_url, Duration::from_secs(2)).expect("concurrent client");
        let mut calls = tokio::task::JoinSet::new();
        for index in 0..16 {
            let client = client.clone();
            calls.spawn(async move { client.create(&format!("task {index}")).await });
        }
        while let Some(result) = calls.join_next().await {
            result.expect("join create").expect("concurrent create");
        }
        let tasks = client
            .list(TaskFilter::default())
            .await
            .expect("concurrent list");
        assert_eq!(tasks.len(), 16);
        assert!(tasks.windows(2).all(|pair| pair[0].id() < pair[1].id()));
        live.stop().await;
    }
}

#[tokio::test]
async fn actix_selection_is_explicitly_incomplete_without_storage_side_effects() {
    let directory = tempfile::tempdir().expect("temporary storage");
    let data = directory.path().join("tasks.db");
    let result = tasks_solution::server::bind(ServerConfig {
        server: ServerKind::Actix,
        backend: BackendKind::Sqlite,
        data: data.clone(),
        host: "127.0.0.1".to_owned(),
        port: 0,
    })
    .await;
    let error = match result {
        Ok(_) => panic!("Actix is Milestone 5"),
        Err(error) => error,
    };
    assert_eq!(
        error.incomplete_capability(),
        Some("Actix Web server adapter (Milestone 5)")
    );
    assert!(!data.exists());
}
