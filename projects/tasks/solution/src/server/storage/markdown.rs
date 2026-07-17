//! Markdown-checklist [`TaskRepository`]: a human-readable single-file backend.
//!
//! Everything lives in one UTF-8 file whose metadata comment records the format
//! version and the next ID. Each mutation loads the whole document, changes it,
//! and saves it. Parsing treats the file as untrusted input and rejects anything
//! malformed instead of guessing a repair. Saving writes a temporary sibling
//! file, flushes it, then atomically renames it over the target so a reader
//! never sees a half-written document. A `Mutex` serializes load-modify-save
//! within this process; cross-process locking and recovery from a crash between
//! filesystem operations are intentionally out of scope.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use tempfile::NamedTempFile;

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskRepository, TaskResult, validate_title};

/// A task store backed by one Markdown checklist file.
#[derive(Debug)]
pub struct MarkdownRepository {
    path: PathBuf,
    // Serializes load-modify-save so concurrent callers in this process cannot
    // interleave and lose writes.
    mutex: Mutex<()>,
}

// The in-memory form of a parsed file: the next ID to allocate plus the tasks.
#[derive(Debug)]
struct Document {
    next_id: i64,
    tasks: Vec<Task>,
}

impl MarkdownRepository {
    /// Opens `path`, initializing a fresh v1 store only when the file does not
    /// exist. An existing file must parse cleanly; an empty or malformed file is
    /// not treated as a new store.
    pub fn open(path: impl AsRef<Path>) -> TaskResult<Self> {
        let path = absolute_target(path.as_ref(), "open markdown")?;
        let repository = Self {
            path,
            mutex: Mutex::new(()),
        };
        if repository.path.exists() {
            repository
                .load()
                .map_err(|error| TaskError::storage("open markdown", error))?;
        } else {
            repository
                .save(&Document {
                    next_id: 1,
                    tasks: Vec::new(),
                })
                .map_err(|error| TaskError::storage("initialize markdown", error))?;
        }
        Ok(repository)
    }

    /// The canonical absolute path of the Markdown file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    // A poisoned lock is reported as a storage error tagged with the operation.
    fn lock(&self, operation: &'static str) -> TaskResult<MutexGuard<'_, ()>> {
        self.mutex.lock().map_err(|_| {
            TaskError::storage(
                operation,
                io::Error::other("Markdown repository lock poisoned"),
            )
        })
    }

    fn load(&self) -> io::Result<Document> {
        let content = fs::read(&self.path)?;
        parse_document(content)
    }

    // Atomic publish: fully write and flush a temporary sibling in the same
    // directory, then rename it over the target. The final directory fsync gives
    // the rename a better chance of surviving to disk, but this is not a crash
    // durability guarantee.
    fn save(&self, document: &Document) -> io::Result<()> {
        let content = serialize(document);
        let parent = self.path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "storage path has no parent")
        })?;
        let mut temporary = NamedTempFile::new_in(parent)?;
        temporary.as_file_mut().write_all(content.as_bytes())?;
        temporary.as_file_mut().flush()?;
        temporary.as_file().sync_all()?;
        temporary
            .into_temp_path()
            .persist(&self.path)
            .map_err(|error| error.error)?;
        sync_directory(parent)
    }
}

impl TaskRepository for MarkdownRepository {
    fn create(&self, title: &str) -> TaskResult<Task> {
        let _guard = self.lock("create task")?;
        let mut document = self
            .load()
            .map_err(|error| TaskError::storage("create task", error))?;
        // Reject rather than wrap around when the ID space is exhausted.
        let next_id = document.next_id.checked_add(1).ok_or_else(|| {
            TaskError::storage(
                "create task",
                io::Error::other("Markdown store has exhausted task IDs"),
            )
        })?;
        let created = Task::from_parts(document.next_id, title, false)?;
        document.next_id = next_id;
        document.tasks.push(created.clone());
        self.save(&document)
            .map_err(|error| TaskError::storage("create task", error))?;
        Ok(created)
    }

    fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>> {
        let _guard = self.lock("list tasks")?;
        let document = self
            .load()
            .map_err(|error| TaskError::storage("list tasks", error))?;
        Ok(document
            .tasks
            .into_iter()
            .filter(|task| {
                filter
                    .completed
                    .is_none_or(|value| task.completed() == value)
            })
            .collect())
    }

    fn get(&self, id: i64) -> TaskResult<Task> {
        let _guard = self.lock("get task")?;
        let document = self
            .load()
            .map_err(|error| TaskError::storage("get task", error))?;
        document
            .tasks
            .into_iter()
            .find(|task| task.id() == id)
            .ok_or_else(|| TaskError::not_found(id))
    }

    fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task> {
        let _guard = self.lock("update task")?;
        let mut document = self
            .load()
            .map_err(|error| TaskError::storage("update task", error))?;
        // Tasks are stored in ID order, so a binary search locates one directly
        // and a miss is a not-found error.
        let index = document
            .tasks
            .binary_search_by_key(&id, Task::id)
            .map_err(|_| TaskError::not_found(id))?;
        let current = &document.tasks[index];
        let updated = Task::from_parts(
            id,
            patch.title.as_deref().unwrap_or(current.title()),
            patch.completed.unwrap_or(current.completed()),
        )?;
        document.tasks[index] = updated.clone();
        self.save(&document)
            .map_err(|error| TaskError::storage("update task", error))?;
        Ok(updated)
    }

    fn delete(&self, id: i64) -> TaskResult<()> {
        let _guard = self.lock("delete task")?;
        let mut document = self
            .load()
            .map_err(|error| TaskError::storage("delete task", error))?;
        let index = document
            .tasks
            .binary_search_by_key(&id, Task::id)
            .map_err(|_| TaskError::not_found(id))?;
        document.tasks.remove(index);
        self.save(&document)
            .map_err(|error| TaskError::storage("delete task", error))
    }
}

// Parses the whole file as untrusted text. Every structural rule (final
// newline, header line, metadata comment, version, checklist markers, positive
// and strictly increasing IDs, valid titles, next-id greater than every ID) is
// checked; a violation is InvalidData rather than a silently skipped line.
fn parse_document(content: Vec<u8>) -> io::Result<Document> {
    if content.is_empty() {
        return Err(invalid_data("Markdown store is empty"));
    }
    let content = String::from_utf8(content)
        .map_err(|_| invalid_data("Markdown store is not valid UTF-8"))?;
    if !content.ends_with('\n') {
        return Err(invalid_data("Markdown store must end with one newline"));
    }
    // Drop the trailing newline so an empty store yields exactly the three
    // header lines rather than a spurious empty final row.
    let lines: Vec<&str> = content[..content.len() - 1].split('\n').collect();
    if lines.len() < 3 || lines[1] != "# Tasks" || !lines[2].is_empty() {
        return Err(invalid_data("Markdown store has an invalid header"));
    }
    let metadata = lines[0]
        .strip_prefix("<!-- rest-task-api:v")
        .and_then(|value| value.strip_suffix(" -->"))
        .ok_or_else(|| invalid_data("Markdown store has invalid metadata"))?;
    let (version, next_id) = metadata
        .split_once(" next-id=")
        .ok_or_else(|| invalid_data("Markdown store has invalid metadata"))?;
    if version != "1" {
        return Err(invalid_data(format!(
            "unsupported Markdown store version {version:?}"
        )));
    }
    let next_id = parse_positive_integer(next_id, "Markdown store has invalid next-id")?;

    let mut tasks = Vec::with_capacity(lines.len().saturating_sub(3));
    let mut previous_id = 0;
    for (index, line) in lines[3..].iter().enumerate() {
        // `- [ ] ` / `- [x] ` are the only accepted markers; the numeric line
        // offset (`index + 4`) points at the human line number in errors.
        let (completed, rest) = if let Some(rest) = line.strip_prefix("- [ ] ") {
            (false, rest)
        } else if let Some(rest) = line.strip_prefix("- [x] ") {
            (true, rest)
        } else {
            return Err(invalid_data(format!(
                "invalid checklist row on line {}",
                index + 4
            )));
        };
        let (id, title) = rest
            .split_once(": ")
            .ok_or_else(|| invalid_data(format!("invalid checklist row on line {}", index + 4)))?;
        let id = parse_positive_integer(id, format!("invalid task ID on line {}", index + 4))?;
        // IDs must be strictly increasing, which also rejects duplicates.
        if id <= previous_id {
            return Err(invalid_data(
                "task IDs must be positive, unique, and ordered",
            ));
        }
        validate_title(title).map_err(|error| {
            invalid_data(format!("invalid title on line {}: {error}", index + 4))
        })?;
        let task = Task::from_parts(id, title, completed).map_err(|error| {
            invalid_data(format!("invalid task on line {}: {error}", index + 4))
        })?;
        tasks.push(task);
        previous_id = id;
    }
    // The recorded next ID must exceed every stored ID, or future allocation
    // could collide with an existing task.
    if next_id <= previous_id {
        return Err(invalid_data("next-id must be greater than every task ID"));
    }
    Ok(Document { next_id, tasks })
}

// Accepts only canonical positive decimals (no leading zero, no sign), so
// `01`, `+1`, or `0` are rejected before `parse` runs.
fn parse_positive_integer(value: &str, message: impl Into<String>) -> io::Result<i64> {
    let canonical = value
        .as_bytes()
        .first()
        .is_some_and(|first| (b'1'..=b'9').contains(first))
        && value.bytes().all(|byte| byte.is_ascii_digit());
    if !canonical {
        return Err(invalid_data(message));
    }
    value.parse::<i64>().map_err(|_| invalid_data(message))
}

// Deterministic serialization: sort by ID, emit the metadata comment and header,
// then one row per task with a single trailing newline. Repeated saves of the
// same tasks produce byte-identical output.
fn serialize(document: &Document) -> String {
    let mut tasks: Vec<&Task> = document.tasks.iter().collect();
    tasks.sort_by_key(|task| task.id());
    let mut content = format!(
        "<!-- rest-task-api:v1 next-id={} -->\n# Tasks\n\n",
        document.next_id
    );
    for task in tasks {
        let marker = if task.completed() { 'x' } else { ' ' };
        content.push_str(&format!("- [{marker}] {}: {}\n", task.id(), task.title()));
    }
    content
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

// Resolves `path` to an absolute path with a stable parent directory; see the
// SQLite backend for the same approach.
fn absolute_target(path: &Path, operation: &'static str) -> TaskResult<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| TaskError::storage(operation, error))?
            .join(path)
    };
    if absolute.exists() {
        fs::canonicalize(&absolute).map_err(|error| TaskError::storage(operation, error))
    } else {
        let name = absolute.file_name().ok_or_else(|| {
            TaskError::storage(
                operation,
                io::Error::new(io::ErrorKind::InvalidInput, "storage path has no file name"),
            )
        })?;
        let parent = absolute.parent().ok_or_else(|| {
            TaskError::storage(
                operation,
                io::Error::new(io::ErrorKind::InvalidInput, "storage path has no parent"),
            )
        })?;
        let parent =
            fs::canonicalize(parent).map_err(|error| TaskError::storage(operation, error))?;
        Ok(parent.join(name))
    }
}

// Flushing the containing directory makes the rename more likely to be durable
// on Unix. Other platforms have no portable equivalent, so this is a no-op.
#[cfg(unix)]
fn sync_directory(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Persisting over an existing directory fails, and the atomic-write path
    // must not leave its temporary sibling behind.
    #[test]
    fn failed_persist_removes_temporary_file() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("tasks.md");
        fs::create_dir(&target).expect("directory target");
        let repository = MarkdownRepository {
            path: target,
            mutex: Mutex::new(()),
        };
        repository
            .save(&Document {
                next_id: 1,
                tasks: Vec::new(),
            })
            .expect_err("persist over a directory must fail");
        let entries = fs::read_dir(directory.path())
            .expect("read directory")
            .map(|entry| entry.expect("directory entry").file_name())
            .collect::<Vec<_>>();
        assert!(
            entries
                .iter()
                .all(|name| !name.to_string_lossy().starts_with(".tmp"))
        );
    }

    // A panic while holding the repository lock surfaces as a storage error on
    // the next operation, not a second panic.
    #[test]
    fn poisoned_lock_is_a_storage_error() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let repository =
            MarkdownRepository::open(directory.path().join("tasks.md")).expect("open repository");
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = repository.mutex.lock().expect("lock repository");
            panic!("poison Markdown lock");
        }));
        let error = repository
            .list(TaskFilter::default())
            .expect_err("poisoned lock must fail");
        assert_eq!(error.storage_operation(), Some("list tasks"));
    }
}
