//! Shared milestone contract suite, included verbatim by every test crate.
//!
//! This single file is pulled in via `#[path = "../../tests/contracts/milestones.rs"]`
//! by the starter, solution, and contracts test crates. Each includes it as a
//! module and aliases the crate under test to `super::subject`, so the exact
//! same assertions run against every implementation. It encodes the observable
//! contract for all five milestones — behavior, error categories, ordering, and
//! HTTP/persistence semantics — without depending on any private internals.

use std::error::Error as _;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use super::subject;
use subject::TaskRepository as _;

// Records each repository call so tests can assert the service delegates the
// expected, normalized arguments in order.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Call {
    Create(String),
    List(subject::TaskFilter),
    Get(i64),
    Update(i64, subject::TaskPatch),
    Delete(i64),
}

// Test double that records every call and returns a fixed stored task, used to
// prove the service forwards normalized inputs without altering results.
struct RecordingRepository {
    calls: Mutex<Vec<Call>>,
    task: subject::Task,
}

impl RecordingRepository {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            task: subject::Task::from_parts(7, "stored", true)
                .expect("construct repository result"),
        }
    }

    fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("lock calls").clone()
    }
}

impl subject::TaskRepository for RecordingRepository {
    fn create(&self, title: &str) -> subject::TaskResult<subject::Task> {
        self.calls
            .lock()
            .expect("lock calls")
            .push(Call::Create(title.to_owned()));
        Ok(self.task.clone())
    }

    fn list(&self, filter: subject::TaskFilter) -> subject::TaskResult<Vec<subject::Task>> {
        self.calls
            .lock()
            .expect("lock calls")
            .push(Call::List(filter));
        Ok(vec![self.task.clone()])
    }

    fn get(&self, id: i64) -> subject::TaskResult<subject::Task> {
        self.calls.lock().expect("lock calls").push(Call::Get(id));
        Ok(self.task.clone())
    }

    fn update(&self, id: i64, patch: subject::TaskPatch) -> subject::TaskResult<subject::Task> {
        self.calls
            .lock()
            .expect("lock calls")
            .push(Call::Update(id, patch));
        Ok(self.task.clone())
    }

    fn delete(&self, id: i64) -> subject::TaskResult<()> {
        self.calls
            .lock()
            .expect("lock calls")
            .push(Call::Delete(id));
        Ok(())
    }
}

// Test double that records the call, then always fails with a fixed not-found
// error, used to prove the service propagates repository errors unchanged.
struct FailingRepository {
    calls: Mutex<Vec<Call>>,
}

impl FailingRepository {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }

    fn fail<T>(&self) -> subject::TaskResult<T> {
        Err(subject::TaskError::not_found(91))
    }

    fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("lock calls").clone()
    }
}

impl subject::TaskRepository for FailingRepository {
    fn create(&self, title: &str) -> subject::TaskResult<subject::Task> {
        self.calls
            .lock()
            .expect("lock calls")
            .push(Call::Create(title.to_owned()));
        self.fail()
    }

    fn list(&self, filter: subject::TaskFilter) -> subject::TaskResult<Vec<subject::Task>> {
        self.calls
            .lock()
            .expect("lock calls")
            .push(Call::List(filter));
        self.fail()
    }

    fn get(&self, id: i64) -> subject::TaskResult<subject::Task> {
        self.calls.lock().expect("lock calls").push(Call::Get(id));
        self.fail()
    }

    fn update(&self, id: i64, patch: subject::TaskPatch) -> subject::TaskResult<subject::Task> {
        self.calls
            .lock()
            .expect("lock calls")
            .push(Call::Update(id, patch));
        self.fail()
    }

    fn delete(&self, id: i64) -> subject::TaskResult<()> {
        self.calls
            .lock()
            .expect("lock calls")
            .push(Call::Delete(id));
        self.fail()
    }
}

/// Milestone 1: domain validation, error taxonomy, and service delegation.
///
/// Exercises title/id/patch/filter normalization, the `TaskError` categories,
/// and that `TaskService` validates before delegating and preserves repository
/// errors verbatim.
pub fn milestone_1_domain_and_contracts() {
    assert_title_rules();
    assert_task_and_validation_rules();
    assert_error_categories();
    assert_service_delegation();
    assert_service_rejects_invalid_input();
    assert_repository_errors_are_preserved();
}

// Pins the exact title-normalization contract: which characters are trimmed at
// the edges, that interior content (including multi-byte scalars) is preserved,
// and that the length bound counts characters rather than bytes.
fn assert_title_rules() {
    let max_ascii = "a".repeat(subject::MAX_TITLE_LENGTH);
    let max_unicode = "界".repeat(subject::MAX_TITLE_LENGTH);
    let success_cases = [
        (
            "trim Unicode whitespace",
            " \u{2003}Learn REST\u{a0} ",
            "Learn REST",
        ),
        ("trim tab", "\tLearn REST\t", "Learn REST"),
        ("trim carriage return", "\rLearn REST\r", "Learn REST"),
        ("trim line feed", "\nLearn REST\n", "Learn REST"),
        (
            "trim Unicode separators",
            "\u{2028}Learn REST\u{2029}",
            "Learn REST",
        ),
        (
            "Unicode scalar values",
            "Καλημέρα 世界 🚀",
            "Καλημέρα 世界 🚀",
        ),
        ("maximum ASCII", max_ascii.as_str(), max_ascii.as_str()),
        (
            "maximum Unicode",
            max_unicode.as_str(),
            max_unicode.as_str(),
        ),
    ];
    for (name, input, expected) in success_cases {
        assert_eq!(
            subject::normalize_title(input).unwrap_or_else(|error| panic!("{name}: {error}")),
            expected,
            "{name}"
        );
    }

    let too_long = format!("{max_unicode}界");
    // Failures fall into three distinct messages: length bounds, "one physical
    // line" (any vertical break), and interior control characters.
    let failure_cases = [
        (
            "empty",
            " \u{2003} ",
            "title must contain between 1 and 120 characters",
        ),
        (
            "too long",
            too_long.as_str(),
            "title must contain between 1 and 120 characters",
        ),
        (
            "line feed",
            "one\ntwo",
            "title must occupy one physical line",
        ),
        (
            "carriage return",
            "one\rtwo",
            "title must occupy one physical line",
        ),
        (
            "vertical tab",
            "one\u{000b}two",
            "title must occupy one physical line",
        ),
        (
            "form feed",
            "one\u{000c}two",
            "title must occupy one physical line",
        ),
        (
            "file separator",
            "one\u{001c}two",
            "title must occupy one physical line",
        ),
        (
            "group separator",
            "one\u{001d}two",
            "title must occupy one physical line",
        ),
        (
            "record separator",
            "one\u{001e}two",
            "title must occupy one physical line",
        ),
        (
            "next line",
            "one\u{0085}two",
            "title must occupy one physical line",
        ),
        (
            "line separator",
            "one\u{2028}two",
            "title must occupy one physical line",
        ),
        (
            "paragraph separator",
            "one\u{2029}two",
            "title must occupy one physical line",
        ),
        (
            "internal tab",
            "one\ttwo",
            "title must not contain control characters",
        ),
        (
            "nul",
            "one\0two",
            "title must not contain control characters",
        ),
    ];
    for (name, input, message) in failure_cases {
        assert_validation(
            subject::normalize_title(input).expect_err(name),
            "title",
            message,
        );
    }

    // `validate_title` checks already-persisted values: it must reject padding
    // rather than silently trimming it the way `normalize_title` does on input.
    assert_validation(
        subject::validate_title(" padded ").expect_err("reject persisted padding"),
        "title",
        "title must not have leading or trailing whitespace",
    );
    subject::validate_title("valid").expect("accept normalized title");
}

fn assert_task_and_validation_rules() {
    for id in [i64::MIN, -1, 0] {
        assert_validation(
            subject::validate_id(id).expect_err("reject non-positive ID"),
            "id",
            "task ID must be a positive integer",
        );
    }
    for id in [1, i64::MAX] {
        subject::validate_id(id).expect("accept positive ID");
    }

    let task = subject::Task::from_parts(1, "Learn REST", false).expect("construct valid task");
    assert_eq!(task.id(), 1);
    assert_eq!(task.title(), "Learn REST");
    assert!(!task.completed());
    assert_eq!(
        serde_json::to_value(&task).expect("serialize task"),
        serde_json::json!({"id": 1, "title": "Learn REST", "completed": false})
    );
    assert_validation(
        subject::Task::from_parts(0, "valid", false).expect_err("reject persisted ID"),
        "id",
        "task ID must be a positive integer",
    );
    assert_validation(
        subject::Task::from_parts(1, " padded ", false)
            .expect_err("reject unnormalized persisted title"),
        "title",
        "title must not have leading or trailing whitespace",
    );

    assert_validation(
        subject::normalize_patch(subject::TaskPatch::default()).expect_err("reject empty patch"),
        "update",
        "update must include title or completed",
    );
    let patch = subject::normalize_patch(subject::TaskPatch {
        title: Some("  updated  ".to_owned()),
        completed: Some(false),
    })
    .expect("normalize patch");
    assert_eq!(patch.title.as_deref(), Some("updated"));
    assert_eq!(patch.completed, Some(false));
    assert_validation(
        subject::validate_patch(&subject::TaskPatch {
            title: Some(" padded ".to_owned()),
            completed: None,
        })
        .expect_err("reject unnormalized patch"),
        "title",
        "title must not have leading or trailing whitespace",
    );

    let filter = subject::normalize_filter(subject::TaskFilter {
        completed: Some(false),
    });
    assert_eq!(filter.completed, Some(false));
    assert_eq!(
        subject::normalize_filter(subject::TaskFilter::default()).completed,
        None
    );
}

// Verifies each `TaskError` constructor exposes its category via the accessor
// used by adapters to map errors to status codes, and formats a stable message.
fn assert_error_categories() {
    let incomplete = subject::TaskError::incomplete("future adapter");
    assert_eq!(incomplete.incomplete_capability(), Some("future adapter"));

    let not_found = subject::TaskError::not_found(42);
    assert_eq!(not_found.to_string(), "task 42 was not found");
    assert_eq!(not_found.not_found_id(), Some(42));

    let storage = subject::TaskError::storage("list", io::Error::other("disk unavailable"));
    assert_eq!(storage.storage_operation(), Some("list"));
    assert_eq!(
        storage.to_string(),
        "task storage list failed: disk unavailable"
    );
    assert_eq!(
        storage.source().expect("storage source").to_string(),
        "disk unavailable"
    );
}

// The service normalizes inputs (trims titles, etc.) before delegating; the
// recorded calls prove the repository receives the cleaned, not raw, arguments.
fn assert_service_delegation() {
    let repository = Arc::new(RecordingRepository::new());
    let service = subject::TaskService::new(repository.clone());
    let expected = subject::Task::from_parts(7, "stored", true).expect("expected task");

    assert_eq!(
        service.create("  stored  ").expect("create"),
        expected,
        "create result"
    );
    assert_eq!(
        service
            .list(subject::TaskFilter {
                completed: Some(false),
            })
            .expect("list"),
        vec![expected.clone()]
    );
    assert_eq!(service.get(7).expect("get"), expected);
    assert_eq!(
        service
            .update(
                7,
                subject::TaskPatch {
                    title: Some("  next  ".to_owned()),
                    completed: Some(false),
                },
            )
            .expect("update"),
        expected
    );
    service.delete(7).expect("delete");

    assert_eq!(
        repository.calls(),
        vec![
            Call::Create("stored".to_owned()),
            Call::List(subject::TaskFilter {
                completed: Some(false)
            }),
            Call::Get(7),
            Call::Update(
                7,
                subject::TaskPatch {
                    title: Some("next".to_owned()),
                    completed: Some(false),
                }
            ),
            Call::Delete(7),
        ]
    );
}

// Invalid inputs must be rejected before any delegation, so the repository sees
// no calls at all — validation is the service's responsibility, not storage's.
fn assert_service_rejects_invalid_input() {
    let repository = Arc::new(RecordingRepository::new());
    let service = subject::TaskService::new(repository.clone());

    let errors = [
        service.create(" ").expect_err("invalid create"),
        service.get(0).expect_err("invalid get"),
        service
            .update(
                0,
                subject::TaskPatch {
                    title: None,
                    completed: Some(true),
                },
            )
            .expect_err("invalid update ID"),
        service
            .update(1, subject::TaskPatch::default())
            .expect_err("invalid empty update"),
        service.delete(-1).expect_err("invalid delete"),
    ];
    for error in errors {
        assert!(
            error.validation_details().is_some(),
            "expected validation error, got {error}"
        );
    }
    assert!(repository.calls().is_empty());
}

// For valid inputs that reach storage, a repository error must surface to the
// caller unchanged (same not-found id), confirming the service adds no rewrapping.
fn assert_repository_errors_are_preserved() {
    type Operation = Box<dyn FnOnce(&subject::TaskService) -> subject::TaskResult<()> + Send>;

    let operations: Vec<(&str, Call, Operation)> = vec![
        (
            "create",
            Call::Create("valid".to_owned()),
            Box::new(|service| service.create("valid").map(|_| ())),
        ),
        (
            "list",
            Call::List(subject::TaskFilter::default()),
            Box::new(|service| service.list(subject::TaskFilter::default()).map(|_| ())),
        ),
        (
            "get",
            Call::Get(1),
            Box::new(|service| service.get(1).map(|_| ())),
        ),
        (
            "update",
            Call::Update(
                1,
                subject::TaskPatch {
                    title: None,
                    completed: Some(false),
                },
            ),
            Box::new(|service| {
                service
                    .update(
                        1,
                        subject::TaskPatch {
                            title: None,
                            completed: Some(false),
                        },
                    )
                    .map(|_| ())
            }),
        ),
        (
            "delete",
            Call::Delete(1),
            Box::new(|service| service.delete(1)),
        ),
    ];

    for (name, expected_call, operation) in operations {
        let repository = Arc::new(FailingRepository::new());
        let service = subject::TaskService::new(repository.clone());
        let error = operation(&service).expect_err(name);
        assert_eq!(error.not_found_id(), Some(91), "{name}");
        assert_eq!(repository.calls(), vec![expected_call], "{name}");
    }
}

// Shared helper: a validation error must expose the (field, message) pair and
// use that same message as its `Display` text.
fn assert_validation(error: subject::TaskError, field: &'static str, message: &'static str) {
    assert_eq!(error.validation_details(), Some((field, message)));
    assert_eq!(error.to_string(), message);
}

/// Milestone 2: both persistence adapters honor the same repository contract.
///
/// Runs an identical CRUD/ordering/validation/restart/concurrency suite against
/// the SQLite and Markdown adapters, then checks each format's on-disk
/// representation, corruption rejection, and path-failure behavior.
pub fn milestone_2_persistence() {
    run_repository_contract("SQLite", ".db", open_sqlite);
    run_repository_contract("Markdown", ".md", open_markdown);
    assert_markdown_format_and_corruption();
    assert_sqlite_schema_and_corruption();
    assert_storage_path_failures();
}

type RepositoryFactory = fn(&Path) -> subject::TaskResult<Arc<dyn subject::TaskRepository>>;

fn open_sqlite(path: &Path) -> subject::TaskResult<Arc<dyn subject::TaskRepository>> {
    subject::storage::sqlite::SqliteRepository::open(path)
        .map(|repository| Arc::new(repository) as Arc<dyn subject::TaskRepository>)
}

fn open_markdown(path: &Path) -> subject::TaskResult<Arc<dyn subject::TaskRepository>> {
    subject::storage::markdown::MarkdownRepository::open(path)
        .map(|repository| Arc::new(repository) as Arc<dyn subject::TaskRepository>)
}

// The backend-agnostic contract every repository must satisfy, run once per
// adapter via a factory so behavior is proven identical across storage formats.
fn run_repository_contract(name: &str, extension: &str, factory: RepositoryFactory) {
    assert_crud_filters_and_ordering(name, extension, factory);
    assert_repository_validation(name, extension, factory);
    assert_missing_ids(name, extension, factory);
    assert_restart_and_id_non_reuse(name, extension, factory);
    assert_concurrent_callers(name, extension, factory);
}

fn repository(
    name: &str,
    extension: &str,
    factory: RepositoryFactory,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    Arc<dyn subject::TaskRepository>,
) {
    let directory = tempfile::tempdir().unwrap_or_else(|error| panic!("{name}: tempdir: {error}"));
    let path = directory.path().join(format!("tasks{extension}"));
    let repository =
        factory(&path).unwrap_or_else(|error| panic!("{name}: open repository: {error}"));
    (directory, path, repository)
}

// Core CRUD behavior plus two subtleties: `list` results are ordered by id, and
// a patch with a field set to `None` leaves that field unchanged (no-op update).
fn assert_crud_filters_and_ordering(name: &str, extension: &str, factory: RepositoryFactory) {
    let (_directory, _path, repository) = repository(name, extension, factory);
    let first = repository
        .create("first")
        .unwrap_or_else(|error| panic!("{name}: create first: {error}"));
    let second = repository
        .create("second")
        .unwrap_or_else(|error| panic!("{name}: create second: {error}"));
    let third = repository
        .create("third")
        .unwrap_or_else(|error| panic!("{name}: create third: {error}"));
    assert_eq!([first.id(), second.id(), third.id()], [1, 2, 3], "{name}");
    assert!(!first.completed(), "{name}");

    let updated = repository
        .update(
            second.id(),
            subject::TaskPatch {
                title: Some("second updated".to_owned()),
                completed: Some(true),
            },
        )
        .unwrap_or_else(|error| panic!("{name}: update: {error}"));
    assert_eq!(updated.title(), "second updated", "{name}");
    assert!(updated.completed(), "{name}");
    assert_eq!(
        task_ids(
            repository
                .list(subject::TaskFilter {
                    completed: Some(false),
                })
                .unwrap_or_else(|error| panic!("{name}: list false: {error}"))
        ),
        vec![1, 3],
        "{name}"
    );
    assert_eq!(
        task_ids(
            repository
                .list(subject::TaskFilter {
                    completed: Some(true),
                })
                .unwrap_or_else(|error| panic!("{name}: list true: {error}"))
        ),
        vec![2],
        "{name}"
    );

    let explicit_false = repository
        .update(
            second.id(),
            subject::TaskPatch {
                title: None,
                completed: Some(false),
            },
        )
        .unwrap_or_else(|error| panic!("{name}: explicit false update: {error}"));
    assert!(!explicit_false.completed(), "{name}");
    let no_op = repository
        .update(
            second.id(),
            subject::TaskPatch {
                title: Some("second updated".to_owned()),
                completed: None,
            },
        )
        .unwrap_or_else(|error| panic!("{name}: no-op update: {error}"));
    assert_eq!(no_op, explicit_false, "{name}");
    assert_eq!(
        repository
            .get(second.id())
            .unwrap_or_else(|error| panic!("{name}: get: {error}")),
        no_op,
        "{name}"
    );
    repository
        .delete(second.id())
        .unwrap_or_else(|error| panic!("{name}: delete: {error}"));
    assert_eq!(
        task_ids(
            repository
                .list(subject::TaskFilter::default())
                .unwrap_or_else(|error| panic!("{name}: list all: {error}"))
        ),
        vec![1, 3],
        "{name}"
    );
}

// Mutations against an unknown id must report not-found for that exact id and
// leave the store untouched.
fn assert_missing_ids(name: &str, extension: &str, factory: RepositoryFactory) {
    let (_directory, _path, repository) = repository(name, extension, factory);
    let operations = [
        repository.get(99).map(|_| ()),
        repository
            .update(
                99,
                subject::TaskPatch {
                    title: Some("missing".to_owned()),
                    completed: Some(false),
                },
            )
            .map(|_| ()),
        repository.delete(99),
    ];
    for error in operations.map(|result| result.expect_err("missing ID")) {
        assert_eq!(error.not_found_id(), Some(99), "{name}: {error}");
    }

    assert!(
        repository
            .list(subject::TaskFilter::default())
            .unwrap_or_else(|error| panic!("{name}: list after missing mutations: {error}"))
            .is_empty(),
        "{name}"
    );
}

// The repository re-validates at the storage boundary: a rejected create must
// not consume an id, and a rejected update must not mutate persisted state.
fn assert_repository_validation(name: &str, extension: &str, factory: RepositoryFactory) {
    let (_directory, _path, repository) = repository(name, extension, factory);
    let create_error = repository
        .create(" padded ")
        .expect_err("invalid repository create");
    assert_validation(
        create_error,
        "title",
        "title must not have leading or trailing whitespace",
    );

    let created = repository
        .create("valid")
        .unwrap_or_else(|error| panic!("{name}: create after validation failure: {error}"));
    assert_eq!(created.id(), 1, "{name}: invalid create consumed an ID");

    let update_error = repository
        .update(
            created.id(),
            subject::TaskPatch {
                title: Some(" padded ".to_owned()),
                completed: None,
            },
        )
        .expect_err("invalid repository update");
    assert_validation(
        update_error,
        "title",
        "title must not have leading or trailing whitespace",
    );
    assert_eq!(
        repository
            .get(created.id())
            .unwrap_or_else(|error| panic!("{name}: get after invalid update: {error}")),
        created,
        "{name}: invalid update changed persisted state"
    );
}

// Reopening the store must preserve data and keep ids monotonic: a deleted id is
// never reused, so the next create allocates a strictly higher id after restart.
fn assert_restart_and_id_non_reuse(name: &str, extension: &str, factory: RepositoryFactory) {
    let directory = tempfile::tempdir().expect("restart tempdir");
    let path = directory.path().join(format!("tasks{extension}"));
    let repository = factory(&path).unwrap_or_else(|error| panic!("{name}: open: {error}"));
    let first = repository
        .create("first")
        .unwrap_or_else(|error| panic!("{name}: create first: {error}"));
    let second = repository
        .create("second")
        .unwrap_or_else(|error| panic!("{name}: create second: {error}"));
    repository
        .delete(first.id())
        .unwrap_or_else(|error| panic!("{name}: delete first: {error}"));
    drop(repository);

    let repository = factory(&path).unwrap_or_else(|error| panic!("{name}: reopen: {error}"));
    assert_eq!(
        repository
            .get(second.id())
            .unwrap_or_else(|error| panic!("{name}: get after restart: {error}")),
        second,
        "{name}"
    );
    repository
        .delete(second.id())
        .unwrap_or_else(|error| panic!("{name}: delete second: {error}"));
    drop(repository);

    let repository =
        factory(&path).unwrap_or_else(|error| panic!("{name}: second reopen: {error}"));
    let third = repository
        .create("third")
        .unwrap_or_else(|error| panic!("{name}: create third: {error}"));
    assert_eq!(third.id(), 3, "{name}");
    assert_eq!(
        repository
            .list(subject::TaskFilter::default())
            .unwrap_or_else(|error| panic!("{name}: final list: {error}")),
        vec![third],
        "{name}"
    );
}

// Many threads sharing one `Arc<dyn TaskRepository>` create concurrently; the
// adapter's internal locking must yield exactly 32 unique, contiguous ids.
fn assert_concurrent_callers(name: &str, extension: &str, factory: RepositoryFactory) {
    let (_directory, _path, repository) = repository(name, extension, factory);
    let handles = (0..32)
        .map(|index| {
            let repository = repository.clone();
            thread::spawn(move || repository.create(&format!("task {index:02}")))
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle
            .join()
            .unwrap_or_else(|_| panic!("{name}: concurrent caller panicked"))
            .unwrap_or_else(|error| panic!("{name}: concurrent create: {error}"));
    }
    let tasks = repository
        .list(subject::TaskFilter::default())
        .unwrap_or_else(|error| panic!("{name}: concurrent list: {error}"));
    assert_eq!(tasks.len(), 32, "{name}");
    assert_eq!(task_ids(tasks), (1..=32).collect::<Vec<_>>(), "{name}");
}

fn task_ids(tasks: Vec<subject::Task>) -> Vec<i64> {
    tasks.into_iter().map(|task| task.id()).collect()
}

// Pins the exact Markdown byte format (versioned metadata header, checklist
// rows, literal titles) and asserts a broad catalog of malformed files is
// rejected as storage errors rather than parsed leniently.
fn assert_markdown_format_and_corruption() {
    let directory = tempfile::tempdir().expect("Markdown format tempdir");
    let path = directory.path().join("tasks.md");
    let repository =
        subject::storage::markdown::MarkdownRepository::open(&path).expect("open Markdown");
    assert!(repository.path().is_absolute());
    assert_eq!(
        fs::read_to_string(&path).expect("read initialized Markdown"),
        "<!-- rest-task-api:v1 next-id=1 -->\n# Tasks\n\n"
    );
    let first = repository
        .create("literal *Markdown*")
        .expect("create first");
    let second = repository.create("second").expect("create second");
    repository
        .update(
            second.id(),
            subject::TaskPatch {
                title: None,
                completed: Some(true),
            },
        )
        .expect("complete second");
    assert_eq!(first.id(), 1);
    assert_eq!(
        fs::read_to_string(&path).expect("read deterministic Markdown"),
        "<!-- rest-task-api:v1 next-id=3 -->\n# Tasks\n\n\
         - [ ] 1: literal *Markdown*\n\
         - [x] 2: second\n"
    );
    drop(repository);

    let malformed: Vec<(&str, Vec<u8>)> = vec![
        ("empty", Vec::new()),
        ("invalid UTF-8", vec![0xff, b'\n']),
        (
            "missing final newline",
            b"<!-- rest-task-api:v1 next-id=1 -->\n# Tasks\n".to_vec(),
        ),
        (
            "extra final newline",
            b"<!-- rest-task-api:v1 next-id=1 -->\n# Tasks\n\n\n".to_vec(),
        ),
        ("missing metadata", b"# Tasks\n\n".to_vec()),
        (
            "unsupported version",
            b"<!-- rest-task-api:v2 next-id=1 -->\n# Tasks\n\n".to_vec(),
        ),
        (
            "noncanonical version",
            b"<!-- rest-task-api:v01 next-id=1 -->\n# Tasks\n\n".to_vec(),
        ),
        (
            "zero next ID",
            b"<!-- rest-task-api:v1 next-id=0 -->\n# Tasks\n\n".to_vec(),
        ),
        (
            "overflowing next ID",
            b"<!-- rest-task-api:v1 next-id=9223372036854775808 -->\n# Tasks\n\n".to_vec(),
        ),
        (
            "invalid heading",
            b"<!-- rest-task-api:v1 next-id=1 -->\n# Task\n\n".to_vec(),
        ),
        (
            "invalid marker",
            b"<!-- rest-task-api:v1 next-id=2 -->\n# Tasks\n\n- [X] 1: title\n".to_vec(),
        ),
        (
            "zero ID",
            b"<!-- rest-task-api:v1 next-id=2 -->\n# Tasks\n\n- [ ] 0: title\n".to_vec(),
        ),
        (
            "duplicate ID",
            b"<!-- rest-task-api:v1 next-id=3 -->\n# Tasks\n\n- [ ] 1: one\n- [x] 1: two\n"
                .to_vec(),
        ),
        (
            "out of order",
            b"<!-- rest-task-api:v1 next-id=3 -->\n# Tasks\n\n- [ ] 2: two\n- [x] 1: one\n"
                .to_vec(),
        ),
        (
            "invalid title",
            b"<!-- rest-task-api:v1 next-id=2 -->\n# Tasks\n\n- [ ] 1: trailing \n".to_vec(),
        ),
        (
            "next ID not greater",
            b"<!-- rest-task-api:v1 next-id=2 -->\n# Tasks\n\n- [ ] 2: title\n".to_vec(),
        ),
        (
            "unexpected blank row",
            b"<!-- rest-task-api:v1 next-id=2 -->\n# Tasks\n\n- [ ] 1: title\n\n".to_vec(),
        ),
    ];
    for (name, content) in malformed {
        let directory = tempfile::tempdir().expect("malformed Markdown tempdir");
        let path = directory.path().join("tasks.md");
        fs::write(&path, content).expect("write malformed Markdown");
        let error = subject::storage::markdown::MarkdownRepository::open(&path).expect_err(name);
        assert_storage_error(&error, name);
    }

    let directory = tempfile::tempdir().expect("Markdown overflow tempdir");
    let path = directory.path().join("tasks.md");
    let content = "<!-- rest-task-api:v1 next-id=9223372036854775807 -->\n# Tasks\n\n";
    fs::write(&path, content).expect("write exhausted Markdown");
    let repository =
        subject::storage::markdown::MarkdownRepository::open(&path).expect("open exhausted store");
    let error = repository
        .create("cannot allocate")
        .expect_err("ID overflow");
    assert_storage_error(&error, "Markdown ID overflow");
    assert_eq!(
        fs::read_to_string(&path).expect("read exhausted store"),
        content
    );
}

// Requires the SQLite schema to match a canonical definition and rejects rows
// that violate domain invariants (bad id, unnormalized title, out-of-range
// completed), plus exhaustion of the id space.
fn assert_sqlite_schema_and_corruption() {
    const SCHEMA: &str = "CREATE TABLE tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        completed INTEGER NOT NULL CHECK (completed IN (0, 1))
    )";

    let directory = tempfile::tempdir().expect("SQLite schema tempdir");
    let path = directory.path().join("tasks.db");
    let connection = rusqlite::Connection::open(&path).expect("create incompatible SQLite");
    connection
        .execute_batch("CREATE TABLE tasks (id INTEGER PRIMARY KEY, title TEXT)")
        .expect("create incompatible schema");
    drop(connection);
    let error =
        subject::storage::sqlite::SqliteRepository::open(&path).expect_err("incompatible schema");
    assert_storage_error(&error, "incompatible SQLite schema");

    for (name, insert) in [
        (
            "invalid persisted ID",
            "INSERT INTO tasks (id, title, completed) VALUES (0, 'valid', 0)",
        ),
        (
            "invalid persisted title",
            "INSERT INTO tasks (id, title, completed) VALUES (1, ' padded ', 0)",
        ),
        (
            "invalid persisted completed",
            "PRAGMA ignore_check_constraints = ON;\
             INSERT INTO tasks (id, title, completed) VALUES (1, 'valid', 2)",
        ),
    ] {
        let directory = tempfile::tempdir().expect("invalid SQLite row tempdir");
        let path = directory.path().join("tasks.db");
        let connection = rusqlite::Connection::open(&path).expect("create SQLite");
        connection
            .execute_batch(SCHEMA)
            .expect("create exact schema");
        connection
            .execute_batch(insert)
            .expect("insert invalid row");
        drop(connection);
        let repository =
            subject::storage::sqlite::SqliteRepository::open(&path).expect("open exact schema");
        assert!(repository.path().is_absolute());
        let error = repository
            .list(subject::TaskFilter::default())
            .expect_err(name);
        assert_storage_error(&error, name);
    }

    let directory = tempfile::tempdir().expect("SQLite overflow tempdir");
    let path = directory.path().join("tasks.db");
    let connection = rusqlite::Connection::open(&path).expect("create exhausted SQLite");
    connection
        .execute_batch(SCHEMA)
        .expect("create exact schema");
    connection
        .execute(
            "INSERT INTO tasks (id, title, completed) VALUES (?1, 'last', 0)",
            rusqlite::params![i64::MAX],
        )
        .expect("advance SQLite sequence");
    connection
        .execute("DELETE FROM tasks", [])
        .expect("delete last row");
    drop(connection);
    let repository =
        subject::storage::sqlite::SqliteRepository::open(&path).expect("open exhausted SQLite");
    let error = repository
        .create("cannot allocate")
        .expect_err("SQLite ID overflow");
    assert_storage_error(&error, "SQLite ID overflow");
    assert!(
        repository
            .list(subject::TaskFilter::default())
            .expect("list after failed SQLite create")
            .is_empty()
    );
}

// Opening a store whose parent directory is missing must fail cleanly without
// leaving stray temp files behind from a partial atomic write.
fn assert_storage_path_failures() {
    let directory = tempfile::tempdir().expect("storage path tempdir");
    let sqlite = directory.path().join("missing").join("tasks.db");
    let markdown = directory.path().join("missing").join("tasks.md");
    let sqlite_error = subject::storage::sqlite::SqliteRepository::open(&sqlite)
        .expect_err("SQLite missing parent");
    let markdown_error = subject::storage::markdown::MarkdownRepository::open(&markdown)
        .expect_err("Markdown missing parent");
    assert_storage_error(&sqlite_error, "SQLite missing parent");
    assert_storage_error(&markdown_error, "Markdown missing parent");
    assert!(
        fs::read_dir(directory.path())
            .expect("read storage path directory")
            .all(|entry| !entry
                .expect("storage path entry")
                .file_name()
                .to_string_lossy()
                .contains(".tmp"))
    );
}

// Shared helper: a storage error must name the failed operation and preserve the
// underlying cause as its error source for diagnostics.
fn assert_storage_error(error: &subject::TaskError, name: &str) {
    assert!(
        error.storage_operation().is_some(),
        "{name}: expected storage operation, got {error}"
    );
    assert!(
        error.source().is_some(),
        "{name}: expected preserved source, got {error}"
    );
}

/// Milestone 3: the Reqwest client and shared HTTP boundary.
///
/// Checks the transport-neutral policy helpers (strict JSON rejecting duplicate
/// keys, id parsing, base-URL normalization, scheme validation) and drives the
/// `HttpBoundary` end to end to confirm it produces a 201 with the JSON envelope.
pub fn milestone_3_client_and_boundary() {
    assert!(subject::api::boundary::strict_json(br#"{"value":1}"#).is_ok());
    assert!(subject::api::boundary::strict_json(br#"{"value":1,"value":2}"#).is_err());
    assert_eq!(subject::api::boundary::parse_id("7").expect("valid ID"), 7);
    assert!(subject::api::boundary::parse_id("+7").is_err());
    assert_eq!(
        subject::client::normalize_base_url("http://EXAMPLE.com/api///").expect("normalized URL"),
        "http://example.com/api"
    );
    assert!(
        subject::client::TaskClient::new("ftp://example.com", std::time::Duration::from_secs(1))
            .is_err()
    );

    let runtime = tokio::runtime::Runtime::new().expect("create Tokio runtime");
    runtime.block_on(async {
        let repository = Arc::new(RecordingRepository::new());
        let boundary = subject::api::boundary::HttpBoundary::new(
            subject::AsyncTaskService::new(subject::TaskService::new(repository)),
            Arc::new(subject::api::boundary::StderrReporter),
        );
        let response = boundary
            .create(
                None,
                Some("application/json"),
                br#"{"title":"  shared boundary  "}"#,
            )
            .await;
        assert_eq!(response.status, 201);
        assert_eq!(
            response.headers,
            vec![(
                "Content-Type".to_owned(),
                subject::api::boundary::JSON_CONTENT_TYPE.to_owned()
            )]
        );
        let value: serde_json::Value =
            serde_json::from_slice(&response.body).expect("decode boundary response");
        assert_eq!(value["id"], 7);
    });
}

/// Milestone 4: the Axum server end to end over a real socket.
///
/// Binds an Axum server on an ephemeral port, drives it with the real Reqwest
/// client, and confirms graceful shutdown via a oneshot channel.
pub fn milestone_4_axum() {
    let runtime = tokio::runtime::Runtime::new().expect("create Tokio runtime");
    runtime.block_on(async {
        let directory = tempfile::tempdir().expect("create server storage");
        let server = subject::server::bind(subject::server::ServerConfig {
            server: subject::server::ServerKind::Axum,
            backend: subject::server::BackendKind::Sqlite,
            data: directory.path().join("tasks.db"),
            host: "127.0.0.1".to_owned(),
            port: 0,
        })
        .await
        .expect("bind Axum server");
        let address = server.local_addr();
        let (shutdown, receiver) = tokio::sync::oneshot::channel();
        let join = tokio::spawn(server.serve(async {
            receiver.await.ok();
        }));
        let client = subject::client::TaskClient::new(
            format!("http://{address}"),
            std::time::Duration::from_secs(2),
        )
        .expect("create Reqwest client");
        let created = client.create("Axum contract").await.expect("create task");
        assert_eq!(created.id(), 1);
        assert_eq!(
            client
                .list(subject::TaskFilter::default())
                .await
                .expect("list tasks"),
            vec![created]
        );
        shutdown.send(()).expect("request shutdown");
        join.await
            .expect("join server")
            .expect("graceful server shutdown");
    });
}

/// Milestone 5: Actix plus cross-adapter interoperability.
///
/// Runs the full matrix of {Axum, Actix} servers over {SQLite, Markdown}
/// backends with one shared client, then restarts each server to confirm data
/// and monotonic ids survive — proving every combination speaks one contract.
pub fn milestone_5_actix_and_interoperability() {
    let runtime = tokio::runtime::Runtime::new().expect("create Tokio runtime");
    runtime.block_on(async {
        for server_kind in [
            subject::server::ServerKind::Axum,
            subject::server::ServerKind::Actix,
        ] {
            for (backend, filename) in [
                (subject::server::BackendKind::Sqlite, "tasks.db"),
                (subject::server::BackendKind::Markdown, "tasks.md"),
            ] {
                let directory = tempfile::tempdir().expect("create matrix storage");
                let data = directory.path().join(filename);
                let config = || subject::server::ServerConfig {
                    server: server_kind,
                    backend,
                    data: data.clone(),
                    host: "127.0.0.1".to_owned(),
                    port: 0,
                };

                let server = subject::server::bind(config())
                    .await
                    .expect("bind matrix server");
                let address = server.local_addr();
                let (shutdown, receiver) = tokio::sync::oneshot::channel();
                let join = tokio::spawn(server.serve(async {
                    receiver.await.ok();
                }));
                let client = subject::client::TaskClient::new(
                    format!("http://{address}"),
                    std::time::Duration::from_secs(2),
                )
                .expect("create shared Reqwest client");
                let created = client.create("matrix task").await.expect("create task");
                assert_eq!(created.id(), 1);
                let completed = client
                    .update(
                        1,
                        subject::TaskPatch {
                            title: None,
                            completed: Some(true),
                        },
                    )
                    .await
                    .expect("complete task");
                assert!(completed.completed());
                client.delete(1).await.expect("delete task");
                shutdown.send(()).expect("request matrix shutdown");
                join.await
                    .expect("join matrix server")
                    .expect("graceful matrix shutdown");

                let restarted = subject::server::bind(config())
                    .await
                    .expect("restart matrix server");
                let address = restarted.local_addr();
                let (shutdown, receiver) = tokio::sync::oneshot::channel();
                let join = tokio::spawn(restarted.serve(async {
                    receiver.await.ok();
                }));
                let client = subject::client::TaskClient::new(
                    format!("http://{address}"),
                    std::time::Duration::from_secs(2),
                )
                .expect("create restart client");
                assert_eq!(
                    client.create("next task").await.expect("monotonic ID").id(),
                    2
                );
                shutdown.send(()).expect("request restart shutdown");
                join.await
                    .expect("join restarted server")
                    .expect("graceful restart shutdown");
            }
        }
    });
}
