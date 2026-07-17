//! SQLite-backed [`TaskRepository`] (stubbed in this scaffold).
//!
//! The backend you build here must serialize all access within the process,
//! run mutations inside a transaction, and use only parameterized statements
//! (never string-built SQL). IDs must stay monotonic and never be reused after
//! deletion, and an unexpected on-disk schema is a storage error rather than
//! something to migrate. Every stored row must map back through the domain
//! constructor so corrupt data cannot enter the core.

use std::path::{Path, PathBuf};

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskRepository, TaskResult};

/// A SQLite task store; holds the resolved database path.
#[derive(Debug)]
pub struct SqliteRepository {
    path: PathBuf,
}

impl SqliteRepository {
    /// Opens (creating if needed) the database at `path` and verifies its
    /// schema; a mismatch must be reported as a storage error.
    pub fn open(_path: impl AsRef<Path>) -> TaskResult<Self> {
        Err(TaskError::incomplete("SQLite connection and schema"))
    }

    /// The resolved database path this repository targets.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
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
