//! Domain types and storage-independent task operations.

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new(value: u64) -> Result<Self, TaskError> {
        if value == 0 {
            Err(TaskError::InvalidId(value))
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

// Deriving `Deserialize` would construct `TaskId(0)` directly and bypass
// `TaskId::new`, so external data is routed through the domain invariant.
impl<'de> Deserialize<'de> for TaskId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    id: TaskId,
    title: String,
    done: bool,
}

impl Task {
    pub fn new(id: TaskId, title: &str) -> Result<Self, TaskError> {
        let title = normalize_title(title)?;
        Ok(Self {
            id,
            title,
            done: false,
        })
    }

    pub(crate) fn validate(self) -> Result<Self, TaskError> {
        let title = normalize_title(&self.title)?;
        Ok(Self { title, ..self })
    }

    #[must_use]
    pub const fn id(&self) -> TaskId {
        self.id
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.done
    }

    pub fn mark_done(&mut self) -> bool {
        if self.done {
            false
        } else {
            self.done = true;
            true
        }
    }
}

fn normalize_title(title: &str) -> Result<String, TaskError> {
    let title = title.trim();
    if title.is_empty() {
        Err(TaskError::InvalidTitle)
    } else {
        Ok(title.to_owned())
    }
}

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("task title must not be empty")]
    InvalidTitle,
    #[error("task id must be positive, got {0}")]
    InvalidId(u64),
    #[error("no task with id {0}")]
    NotFound(TaskId),
    #[error("invalid task storage: {0}")]
    InvalidStorage(String),
    #[error("cannot access {}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot decode {}: {source}", path.display())]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

impl TaskError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn json(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.into(),
            source,
        }
    }
}

pub trait TaskStore {
    fn tasks(&self) -> &[Task];
    fn add(&mut self, title: &str) -> Result<Task, TaskError>;
    fn complete(&mut self, id: TaskId) -> Result<Task, TaskError>;
    fn remove(&mut self, id: TaskId) -> Result<Task, TaskError>;
}

pub struct TaskManager<S> {
    store: S,
}

impl<S: TaskStore> TaskManager<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub fn add(&mut self, title: &str) -> Result<Task, TaskError> {
        self.store.add(title)
    }

    pub fn list(&self, include_done: bool) -> Vec<&Task> {
        self.store
            .tasks()
            .iter()
            .filter(|task| include_done || !task.is_done())
            .collect()
    }

    pub fn complete(&mut self, id: TaskId) -> Result<Task, TaskError> {
        self.store.complete(id)
    }

    pub fn remove(&mut self, id: TaskId) -> Result<Task, TaskError> {
        self.store.remove(id)
    }

    pub fn into_store(self) -> S {
        self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryTaskStore;

    #[test]
    fn task_preserves_invariants_and_completion_state() {
        let id = TaskId::new(1).expect("positive id");
        let mut task = Task::new(id, "  Learn Rust  ").expect("valid title");
        assert_eq!(task.title(), "Learn Rust");
        assert!(task.mark_done());
        assert!(!task.mark_done());
        assert!(task.is_done());
    }

    #[test]
    fn deserialization_cannot_bypass_task_id_invariant() {
        let result = serde_json::from_str::<Task>(r#"{"id":0,"title":"Invalid","done":false}"#);
        assert!(result.is_err());
    }

    #[test]
    fn manager_filters_without_changing_storage() {
        let mut manager = TaskManager::new(InMemoryTaskStore::new());
        let first = manager.add("Pending").expect("add");
        let second = manager.add("Done").expect("add");
        manager.complete(second.id()).expect("complete");

        assert_eq!(manager.list(false), vec![&first]);
        assert_eq!(manager.list(true).len(), 2);
    }
}
