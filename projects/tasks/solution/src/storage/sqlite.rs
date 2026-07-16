use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskRepository, TaskResult};

#[derive(Debug)]
pub struct SqliteRepository {
    path: PathBuf,
}

impl SqliteRepository {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn connect(&self) -> TaskResult<Connection> {
        Err(TaskError::incomplete("SQLite connection and schema"))
    }
}

impl TaskRepository for SqliteRepository {
    fn create(&self, _title: &str) -> TaskResult<Task> {
        Err(TaskError::incomplete("SQLite create"))
    }

    fn list(&self, _filter: TaskFilter) -> TaskResult<Vec<Task>> {
        Err(TaskError::incomplete("SQLite list"))
    }

    fn get(&self, _id: i64) -> TaskResult<Task> {
        Err(TaskError::incomplete("SQLite get"))
    }

    fn update(&self, _id: i64, _patch: TaskPatch) -> TaskResult<Task> {
        Err(TaskError::incomplete("SQLite update"))
    }

    fn delete(&self, _id: i64) -> TaskResult<()> {
        Err(TaskError::incomplete("SQLite delete"))
    }
}
