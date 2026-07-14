use std::fs;
use task_manager::cli::{Command, execute};
use task_manager::domain::{TaskError, TaskId, TaskManager, TaskStore};
use task_manager::storage::{InMemoryTaskStore, JsonFileTaskStore};

fn assert_store_contract(mut store: impl TaskStore) {
    let first = store.add("First").expect("first add");
    let second = store.add("Second").expect("second add");
    assert_eq!((first.id().get(), second.id().get()), (1, 2));
    assert_eq!(store.tasks().len(), 2);

    let completed = store.complete(first.id()).expect("complete");
    assert!(completed.is_done());
    let removed = store.remove(second.id()).expect("remove");
    assert_eq!(removed.title(), "Second");

    assert!(matches!(
        store.complete(TaskId::new(999).expect("id")),
        Err(TaskError::NotFound(_))
    ));
}

#[test]
fn in_memory_store_satisfies_contract() {
    assert_store_contract(InMemoryTaskStore::new());
}

#[test]
fn file_store_satisfies_contract_and_persists_monotonic_ids() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("tasks.json");
    assert_store_contract(JsonFileTaskStore::open(&path).expect("open"));

    let mut reopened = JsonFileTaskStore::open(&path).expect("reopen");
    assert_eq!(reopened.tasks().len(), 1);
    assert!(reopened.tasks()[0].is_done());
    assert_eq!(reopened.add("Third").expect("add").id().get(), 3);
}

#[test]
fn file_store_rejects_inconsistent_data() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("invalid.json");
    fs::write(
        &path,
        r#"{
  "version": 1,
  "next_id": 1,
  "tasks": [{"id": 1, "title": "Existing", "done": false}]
}"#,
    )
    .expect("write fixture");

    assert!(matches!(
        JsonFileTaskStore::open(&path),
        Err(TaskError::InvalidStorage(_))
    ));

    let zero_id_path = directory.path().join("zero-id.json");
    fs::write(
        &zero_id_path,
        r#"{
  "version": 1,
  "next_id": 1,
  "tasks": [{"id": 0, "title": "Invalid", "done": false}]
}"#,
    )
    .expect("write fixture");
    assert!(matches!(
        JsonFileTaskStore::open(zero_id_path),
        Err(TaskError::Json { .. })
    ));
}

#[test]
fn command_execution_is_testable_without_a_process() {
    let mut manager = TaskManager::new(InMemoryTaskStore::new());
    let added = execute(
        &mut manager,
        Command::Add {
            title: String::from("From CLI"),
        },
    )
    .expect("add");
    assert_eq!(added, "Added task #1: From CLI");

    let listing = execute(
        &mut manager,
        Command::List {
            pending_only: false,
        },
    )
    .expect("list");
    assert_eq!(listing, "[ ] #1 From CLI");

    let completed = execute(&mut manager, Command::Complete { id: 1 }).expect("complete");
    assert_eq!(completed, "Completed task #1: From CLI");

    let pending = execute(&mut manager, Command::List { pending_only: true }).expect("list");
    assert_eq!(pending, "No tasks yet.");
}
