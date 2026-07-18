//! The application service and the repository port it depends on.
//!
//! [`TaskService`] holds the synchronous business logic: it validates and
//! normalizes input, then delegates storage to an injected [`TaskRepository`].
//! [`AsyncTaskService`] (aliased [`TaskApplication`]) is the async facade the
//! HTTP boundary calls; its contract is to move each synchronous repository
//! call off the async runtime so a blocking database or file operation cannot
//! stall it. The repository is injected as `Arc<dyn TaskRepository>`, which is
//! how SQLite and Markdown backends become interchangeable. In this scaffold
//! the method bodies return visible typed failures until each milestone is built.

use std::sync::Arc;

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskResult};

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

    pub async fn create(&self, _title: String) -> TaskResult<Task> {
        Err(TaskError::incomplete("async application create"))
    }

    pub async fn list(&self, _filter: TaskFilter) -> TaskResult<Vec<Task>> {
        Err(TaskError::incomplete("async application list"))
    }

    pub async fn get(&self, _id: i64) -> TaskResult<Task> {
        Err(TaskError::incomplete("async application get"))
    }

    pub async fn update(&self, _id: i64, _patch: TaskPatch) -> TaskResult<Task> {
        Err(TaskError::incomplete("async application update"))
    }

    pub async fn delete(&self, _id: i64) -> TaskResult<()> {
        Err(TaskError::incomplete("async application delete"))
    }
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

    // Contract: each method must validate and normalize input in the core before
    // touching storage, so every backend receives already-valid values.
    pub fn create(&self, _title: &str) -> TaskResult<Task> {
        Err(TaskError::incomplete("application create"))
    }

    pub fn list(&self, _filter: TaskFilter) -> TaskResult<Vec<Task>> {
        Err(TaskError::incomplete("application list"))
    }

    pub fn get(&self, _id: i64) -> TaskResult<Task> {
        Err(TaskError::incomplete("application get"))
    }

    pub fn update(&self, _id: i64, _patch: TaskPatch) -> TaskResult<Task> {
        Err(TaskError::incomplete("application update"))
    }

    pub fn delete(&self, _id: i64) -> TaskResult<()> {
        Err(TaskError::incomplete("application delete"))
    }
}
