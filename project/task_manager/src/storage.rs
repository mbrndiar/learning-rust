//! In-memory and atomic JSON task storage strategies.

use crate::domain::{Task, TaskError, TaskId, TaskStore};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORAGE_VERSION: u8 = 1;

#[derive(Debug, Clone)]
pub struct InMemoryTaskStore {
    tasks: Vec<Task>,
    next_id: u64,
}

impl InMemoryTaskStore {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    fn from_parts(tasks: Vec<Task>, next_id: u64) -> Result<Self, TaskError> {
        if next_id == 0 {
            return Err(TaskError::InvalidStorage(String::from(
                "next_id must be positive",
            )));
        }

        let tasks: Vec<Task> = tasks
            .into_iter()
            .map(Task::validate)
            .collect::<Result<_, _>>()?;
        let ids: BTreeSet<_> = tasks.iter().map(Task::id).collect();
        if ids.len() != tasks.len() {
            return Err(TaskError::InvalidStorage(String::from(
                "task ids must be unique",
            )));
        }

        let maximum_id = ids.last().map_or(0, |id| id.get());
        if next_id <= maximum_id {
            return Err(TaskError::InvalidStorage(format!(
                "next_id {next_id} must exceed maximum task id {maximum_id}"
            )));
        }

        Ok(Self { tasks, next_id })
    }

    fn find_index(&self, id: TaskId) -> Result<usize, TaskError> {
        self.tasks
            .iter()
            .position(|task| task.id() == id)
            .ok_or(TaskError::NotFound(id))
    }
}

impl Default for InMemoryTaskStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskStore for InMemoryTaskStore {
    fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    fn add(&mut self, title: &str) -> Result<Task, TaskError> {
        let id = TaskId::new(self.next_id)?;
        let task = Task::new(id, title)?;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| TaskError::InvalidStorage(String::from("task id space exhausted")))?;
        self.tasks.push(task.clone());
        Ok(task)
    }

    fn complete(&mut self, id: TaskId) -> Result<Task, TaskError> {
        let index = self.find_index(id)?;
        self.tasks[index].mark_done();
        Ok(self.tasks[index].clone())
    }

    fn remove(&mut self, id: TaskId) -> Result<Task, TaskError> {
        let index = self.find_index(id)?;
        Ok(self.tasks.remove(index))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageFile {
    version: u8,
    next_id: u64,
    tasks: Vec<Task>,
}

pub struct JsonFileTaskStore {
    path: PathBuf,
    state: InMemoryTaskStore,
}

impl JsonFileTaskStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, TaskError> {
        let path = path.into();
        if !path.exists() {
            return Ok(Self {
                path,
                state: InMemoryTaskStore::new(),
            });
        }

        let contents = fs::read_to_string(&path).map_err(|error| TaskError::io(&path, error))?;
        let file: StorageFile =
            serde_json::from_str(&contents).map_err(|error| TaskError::json(&path, error))?;
        if file.version != STORAGE_VERSION {
            return Err(TaskError::InvalidStorage(format!(
                "unsupported version {}; expected {STORAGE_VERSION}",
                file.version
            )));
        }

        Ok(Self {
            path,
            state: InMemoryTaskStore::from_parts(file.tasks, file.next_id)?,
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn save_state(&self, state: &InMemoryTaskStore) -> Result<(), TaskError> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|error| TaskError::io(parent, error))?;

        let payload = serde_json::to_vec_pretty(&StorageFile {
            version: STORAGE_VERSION,
            next_id: state.next_id,
            tasks: state.tasks.clone(),
        })
        .map_err(|error| TaskError::json(&self.path, error))?;

        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .map_err(|error| TaskError::io(parent, error))?;
        temporary
            .write_all(&payload)
            .and_then(|()| temporary.write_all(b"\n"))
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|error| TaskError::io(temporary.path(), error))?;
        temporary
            .persist(&self.path)
            .map_err(|error| TaskError::io(&self.path, error.error))?;
        Ok(())
    }

    fn commit<R>(
        &mut self,
        operation: impl FnOnce(&mut InMemoryTaskStore) -> Result<R, TaskError>,
    ) -> Result<R, TaskError> {
        let mut candidate = self.state.clone();
        let result = operation(&mut candidate)?;
        self.save_state(&candidate)?;
        self.state = candidate;
        Ok(result)
    }
}

impl TaskStore for JsonFileTaskStore {
    fn tasks(&self) -> &[Task] {
        self.state.tasks()
    }

    fn add(&mut self, title: &str) -> Result<Task, TaskError> {
        self.commit(|state| state.add(title))
    }

    fn complete(&mut self, id: TaskId) -> Result<Task, TaskError> {
        self.commit(|state| state.complete(id))
    }

    fn remove(&mut self, id: TaskId) -> Result<Task, TaskError> {
        self.commit(|state| state.remove(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_ids_and_inconsistent_next_id() {
        let first = Task::new(TaskId::new(1).expect("id"), "First").expect("task");
        let duplicate = first.clone();
        assert!(InMemoryTaskStore::from_parts(vec![first, duplicate], 2).is_err());

        let task = Task::new(TaskId::new(3).expect("id"), "Third").expect("task");
        assert!(InMemoryTaskStore::from_parts(vec![task], 3).is_err());
    }
}
