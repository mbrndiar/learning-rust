//! In-memory and atomic JSON task storage strategies.
//!
//! [`InMemoryTaskStore`] is the source of truth for the store invariants
//! (unique ids and a `next_id` that never collides). [`JsonFileTaskStore`]
//! adds single-writer JSON persistence by writing to a temp file and atomically
//! renaming it into place. File and directory synchronization improve crash
//! durability on Unix, but exact guarantees depend on the OS and filesystem.

use crate::domain::{Task, TaskError, TaskId, TaskStore};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORAGE_VERSION: u8 = 1;

// A bare filename has an empty parent; treat that as the current directory so
// `create_dir_all` and temp-file creation have a real directory to target.
fn parent_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

/// A `Vec`-backed store.
///
/// Invariants, upheld by the internal `from_parts` constructor and
/// [`TaskStore::add`]: task ids are unique and `next_id` is strictly greater than
/// every existing id, so freshly allocated ids never collide.
#[derive(Debug, Clone)]
pub struct InMemoryTaskStore {
    tasks: Vec<Task>,
    next_id: u64,
}

impl InMemoryTaskStore {
    /// Creates an empty store whose first allocated id will be 1.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    // Rebuilds a store from persisted parts, re-validating every invariant
    // because the data came from an untrusted file: titles are re-checked, ids
    // must be unique, and `next_id` must exceed the largest id present.
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
        // Collecting ids into a set both detects duplicates (via the length
        // comparison) and exposes the maximum through `last`.
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
        // Advance `next_id` only after the task validates; `checked_add` guards
        // the (practically unreachable) overflow of the id space.
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

/// On-disk envelope: a `version` tag lets future formats be detected and
/// rejected, alongside the persisted `next_id` and task list.
#[derive(Debug, Serialize, Deserialize)]
struct StorageFile {
    version: u8,
    next_id: u64,
    tasks: Vec<Task>,
}

/// A versioned JSON store intended for one writer process at a time.
///
/// Each save atomically replaces the destination, but the store does not lock
/// against another process loading and overwriting the same file concurrently.
pub struct JsonFileTaskStore {
    path: PathBuf,
    state: InMemoryTaskStore,
}

impl JsonFileTaskStore {
    /// Opens the store at `path`, returning an empty store when the file does
    /// not yet exist. An existing file must match `STORAGE_VERSION` and pass
    /// the [`InMemoryTaskStore`] invariant checks, otherwise a [`TaskError`] is
    /// returned.
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

    /// Returns the file path this store persists to.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    // Atomically overwrites the destination file. The payload is written to a
    // temp file in the same directory, fsynced, then renamed over the target so
    // readers see either the old or new complete payload. On Unix, syncing the
    // parent directory requests persistence of the renamed entry; exact crash
    // durability still depends on the filesystem.
    fn save_state(&self, state: &InMemoryTaskStore) -> Result<(), TaskError> {
        let parent = parent_directory(&self.path);
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
        #[cfg(unix)]
        {
            // Persist the directory entry as well as the file contents.
            fs::File::open(parent)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| TaskError::io(parent, error))?;
        }
        Ok(())
    }

    // Applies a mutation to a *clone* of the current state and persists it
    // before adopting it. Failures before the rename leave `self.state`
    // untouched and the old file intact. A post-rename directory-sync failure
    // is reported, but the file may already contain the candidate state.
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

    #[test]
    fn bare_storage_filename_uses_current_directory() {
        assert_eq!(parent_directory(Path::new("tasks.json")), Path::new("."));
        assert_eq!(
            parent_directory(Path::new("data/tasks.json")),
            Path::new("data")
        );
    }
}
