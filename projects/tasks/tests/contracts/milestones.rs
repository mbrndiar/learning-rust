use std::error::Error as _;
use std::io;
use std::sync::{Arc, Mutex};

use super::subject;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Call {
    Create(String),
    List(subject::TaskFilter),
    Get(i64),
    Update(i64, subject::TaskPatch),
    Delete(i64),
}

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

pub fn milestone_1_domain_and_contracts() {
    assert_title_rules();
    assert_task_and_validation_rules();
    assert_error_categories();
    assert_service_delegation();
    assert_service_rejects_invalid_input();
    assert_repository_errors_are_preserved();
}

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

fn assert_validation(error: subject::TaskError, field: &'static str, message: &'static str) {
    assert_eq!(error.validation_details(), Some((field, message)));
    assert_eq!(error.to_string(), message);
}

pub fn milestone_2_persistence() {
    panic!("milestone 2 incomplete: implement SQLite and Markdown repositories");
}

pub fn milestone_3_client_and_boundary() {
    panic!("milestone 3 incomplete: implement Reqwest and shared HTTP boundaries");
}

pub fn milestone_4_axum() {
    panic!("milestone 4 incomplete: implement the Axum server adapter");
}

pub fn milestone_5_actix_and_interoperability() {
    panic!("milestone 5 incomplete: implement Actix Web and interoperability");
}
