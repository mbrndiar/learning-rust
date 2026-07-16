use std::sync::Arc;

use crate::{
    Task, TaskFilter, TaskPatch, TaskResult, normalize_filter, normalize_patch, normalize_title,
    validate_id,
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
