//! Markdown-checklist [`TaskRepository`] (stubbed in this scaffold).
//!
//! The backend you build here keeps everything in one UTF-8 file whose metadata
//! comment records the format version and the next ID. Each mutation should load
//! the whole document, change it, and save it. Parsing must treat the file as
//! untrusted input and reject anything malformed instead of guessing a repair.
//! Saving should replace the file atomically so a reader never sees a
//! half-written document, and a lock must serialize load-modify-save within this
//! process; cross-process locking and crash recovery are out of scope.

use std::path::{Path, PathBuf};

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskRepository, TaskResult};

/// A task store backed by one Markdown checklist file.
#[derive(Debug)]
pub struct MarkdownRepository {
    path: PathBuf,
}

impl MarkdownRepository {
    /// Opens `path`, initializing a fresh store only when the file does not
    /// exist; an existing file must parse cleanly.
    pub fn open(_path: impl AsRef<Path>) -> TaskResult<Self> {
        Err(TaskError::incomplete("Markdown initialization"))
    }

    /// The resolved file path this repository targets.
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
