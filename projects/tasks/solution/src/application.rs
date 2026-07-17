use std::sync::Arc;

use crate::{
    Task, TaskError, TaskFilter, TaskPatch, TaskResult, normalize_filter, normalize_patch,
    normalize_title, validate_id,
};

pub trait TaskRepository: Send + Sync {
    fn create(&self, title: &str) -> TaskResult<Task>;
    fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>>;
    fn get(&self, id: i64) -> TaskResult<Task>;
    fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task>;
    fn delete(&self, id: i64) -> TaskResult<()>;
}

#[derive(Clone)]
pub struct TaskService {
    repository: Arc<dyn TaskRepository>,
}

#[derive(Clone)]
pub struct AsyncTaskService {
    service: TaskService,
}

pub type TaskApplication = AsyncTaskService;

impl AsyncTaskService {
    #[must_use]
    pub const fn new(service: TaskService) -> Self {
        Self { service }
    }

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
    #[must_use]
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self { repository }
    }

    #[must_use]
    pub fn repository(&self) -> &Arc<dyn TaskRepository> {
        &self.repository
    }

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
