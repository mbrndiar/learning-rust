use std::path::{Path, PathBuf};

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskRepository, TaskResult};

#[derive(Debug)]
pub struct MarkdownRepository {
    path: PathBuf,
}

impl MarkdownRepository {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl TaskRepository for MarkdownRepository {
    fn create(&self, _title: &str) -> TaskResult<Task> {
        Err(TaskError::incomplete("Markdown create"))
    }

    fn list(&self, _filter: TaskFilter) -> TaskResult<Vec<Task>> {
        Err(TaskError::incomplete("Markdown list"))
    }

    fn get(&self, _id: i64) -> TaskResult<Task> {
        Err(TaskError::incomplete("Markdown get"))
    }

    fn update(&self, _id: i64, _patch: TaskPatch) -> TaskResult<Task> {
        Err(TaskError::incomplete("Markdown update"))
    }

    fn delete(&self, _id: i64) -> TaskResult<()> {
        Err(TaskError::incomplete("Markdown delete"))
    }
}
