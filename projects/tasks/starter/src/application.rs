use std::sync::Arc;

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskResult};

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

impl AsyncTaskService {
    #[must_use]
    pub const fn new(service: TaskService) -> Self {
        Self { service }
    }

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
    #[must_use]
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self { repository }
    }

    #[must_use]
    pub fn repository(&self) -> &Arc<dyn TaskRepository> {
        &self.repository
    }

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
