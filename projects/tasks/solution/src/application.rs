//! The application service and the repository port it depends on.
//!
//! [`TaskService`] holds the synchronous business logic: it validates and
//! normalizes input, then delegates storage to an injected [`TaskRepository`].
//! [`AsyncTaskService`] (aliased [`TaskApplication`]) is the async facade the
//! HTTP boundary calls; it moves each synchronous repository call onto Tokio's
//! blocking pool so a blocking `rusqlite` or file operation never stalls the
//! async runtime. The repository is injected as `Arc<dyn TaskRepository>`, which
//! is how SQLite and Markdown backends become interchangeable.

use std::sync::Arc;

use crate::{
    Task, TaskError, TaskFilter, TaskPatch, TaskResult, normalize_filter, normalize_patch,
    normalize_title, validate_id,
};

/// The narrow persistence port implemented by each storage backend.
///
/// The vocabulary is domain values, never framework requests or database rows.
/// `Send + Sync` lets one repository be shared across worker threads behind an
/// `Arc`.
pub trait TaskRepository: Send + Sync {
    /// Persists a new, incomplete task with the given normalized title.
    fn create(&self, title: &str) -> TaskResult<Task>;
    /// Returns all tasks matching `filter`, ordered by ascending ID.
    fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>>;
    /// Returns one task, or [`TaskError::NotFound`] if the ID is absent.
    fn get(&self, id: i64) -> TaskResult<Task>;
    /// Applies a normalized patch to one task and returns the updated value.
    fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task>;
    /// Deletes one task, or reports [`TaskError::NotFound`] if it is absent.
    fn delete(&self, id: i64) -> TaskResult<()>;
}

/// Synchronous application service: validation plus delegation to a repository.
#[derive(Clone)]
pub struct TaskService {
    repository: Arc<dyn TaskRepository>,
}

/// Async facade over [`TaskService`] used by the HTTP adapters.
#[derive(Clone)]
pub struct AsyncTaskService {
    service: TaskService,
}

/// The async service the servers run against.
pub type TaskApplication = AsyncTaskService;

impl AsyncTaskService {
    /// Wraps a synchronous [`TaskService`] in the async facade.
    #[must_use]
    pub const fn new(service: TaskService) -> Self {
        Self { service }
    }

    /// Borrows the underlying synchronous service.
    #[must_use]
    pub const fn service(&self) -> &TaskService {
        &self.service
    }

    pub async fn create(&self, title: String) -> TaskResult<Task> {
        let service = self.service.clone();
        run_blocking("create task", move || service.create(&title)).await
    }

    pub async fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>> {
        let service = self.service.clone();
        run_blocking("list tasks", move || service.list(filter)).await
    }

    pub async fn get(&self, id: i64) -> TaskResult<Task> {
        let service = self.service.clone();
        run_blocking("get task", move || service.get(id)).await
    }

    pub async fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task> {
        let service = self.service.clone();
        run_blocking("update task", move || service.update(id, patch)).await
    }

    pub async fn delete(&self, id: i64) -> TaskResult<()> {
        let service = self.service.clone();
        run_blocking("delete task", move || service.delete(id)).await
    }
}

// Runs a synchronous repository operation on Tokio's blocking pool. A join
// failure (for example a panicking repository) becomes an `Internal` error so
// the async boundary never leaks a raw `JoinError` or unwinds a worker.
async fn run_blocking<T, F>(operation: &'static str, action: F) -> TaskResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> TaskResult<T> + Send + 'static,
{
    tokio::task::spawn_blocking(action)
        .await
        .map_err(|error| TaskError::internal(operation, error))?
}

impl TaskService {
    /// Builds a service over an injected repository.
    #[must_use]
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self { repository }
    }

    /// Borrows the injected repository.
    #[must_use]
    pub fn repository(&self) -> &Arc<dyn TaskRepository> {
        &self.repository
    }

    // Each method validates and normalizes input in the core before touching
    // storage, so every backend receives values that already satisfy the rules.
    pub fn create(&self, title: &str) -> TaskResult<Task> {
        let title = normalize_title(title)?;
        self.repository.create(&title)
    }

    pub fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>> {
        self.repository.list(normalize_filter(filter))
    }

    pub fn get(&self, id: i64) -> TaskResult<Task> {
        validate_id(id)?;
        self.repository.get(id)
    }

    pub fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task> {
        validate_id(id)?;
        let patch = normalize_patch(patch)?;
        self.repository.update(id, patch)
    }

    pub fn delete(&self, id: i64) -> TaskResult<()> {
        validate_id(id)?;
        self.repository.delete(id)
    }
}
