//! SQLite-backed [`TaskRepository`] using `rusqlite` with bundled SQLite.
//!
//! One process owns one `Connection` guarded by a `Mutex`, so all access is
//! serialized in-process. Mutations run inside a transaction, every statement is
//! parameterized (no string-built SQL), and rows are mapped explicitly back into
//! validated domain values. Deleting rows never resets the `AUTOINCREMENT`
//! sequence, which is how IDs stay monotonic and are not reused. This project
//! does not use migrations, WAL tuning, connection pools, or an ORM.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{Connection, Error as SqliteError, OptionalExtension, Transaction, params};

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskRepository, TaskResult, validate_title};

// The exact schema this repository expects. `AUTOINCREMENT` guarantees IDs are
// never reused after deletion; the CHECK keeps `completed` a strict 0/1 Boolean.
const SCHEMA: &str = r"CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    completed INTEGER NOT NULL CHECK (completed IN (0, 1))
)";

// Idempotent form used on open, so a fresh and an already-initialized database
// are both accepted. The stored schema is then compared against `SCHEMA`.
const INITIALIZE_SCHEMA: &str = r"CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    completed INTEGER NOT NULL CHECK (completed IN (0, 1))
)";

/// A SQLite task store owning one guarded connection.
#[derive(Debug)]
pub struct SqliteRepository {
    path: PathBuf,
    connection: Mutex<Connection>,
}

impl SqliteRepository {
    /// Opens (creating if needed) the database at `path` and verifies its
    /// schema. A schema that does not match [`struct@SqliteRepository`]'s
    /// expectation is a storage error rather than something to migrate.
    pub fn open(path: impl AsRef<Path>) -> TaskResult<Self> {
        let path = absolute_target(path.as_ref(), "open sqlite")?;
        let connection =
            Connection::open(&path).map_err(|error| TaskError::storage("open sqlite", error))?;
        // Wait briefly rather than failing immediately if the file is locked.
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|error| TaskError::storage("configure sqlite", error))?;
        initialize_schema(&connection)?;
        Ok(Self {
            path,
            connection: Mutex::new(connection),
        })
    }

    /// The canonical absolute path of the database file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    // A poisoned lock (a previous panic while holding it) is reported as a
    // storage error tagged with the operation, never silently ignored.
    fn lock(&self, operation: &'static str) -> TaskResult<MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|_| {
            TaskError::storage(
                operation,
                io::Error::other("SQLite connection lock poisoned"),
            )
        })
    }
}

impl TaskRepository for SqliteRepository {
    fn create(&self, title: &str) -> TaskResult<Task> {
        validate_title(title)?;
        let mut connection = self.lock("create task")?;
        // Insert and read back inside one transaction so the returned task
        // reflects exactly what was committed, or nothing is committed at all.
        let transaction = connection
            .transaction()
            .map_err(|error| TaskError::storage("create task", error))?;
        transaction
            .execute(
                "INSERT INTO tasks (title, completed) VALUES (?1, ?2)",
                params![title, 0],
            )
            .map_err(|error| TaskError::storage("create task", error))?;
        let id = transaction.last_insert_rowid();
        let created = query_task(&transaction, id, "create task")?;
        transaction
            .commit()
            .map_err(|error| TaskError::storage("create task", error))?;
        Ok(created)
    }

    fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>> {
        let connection = self.lock("list tasks")?;
        // Choose a filtered or unfiltered statement; both order by ID so results
        // are deterministic.
        let (statement, completed) = match filter.completed {
            Some(value) => (
                "SELECT id, title, completed FROM tasks \
                 WHERE completed = ?1 ORDER BY id ASC",
                Some(bool_integer(value)),
            ),
            None => (
                "SELECT id, title, completed FROM tasks ORDER BY id ASC",
                None,
            ),
        };
        let mut prepared = connection
            .prepare(statement)
            .map_err(|error| TaskError::storage("list tasks", error))?;
        let mut rows = match completed {
            Some(value) => prepared.query(params![value]),
            None => prepared.query([]),
        }
        .map_err(|error| TaskError::storage("list tasks", error))?;
        let mut tasks = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|error| TaskError::storage("list tasks", error))?
        {
            let parts = read_parts(row).map_err(|error| TaskError::storage("list tasks", error))?;
            tasks.push(task_from_parts(parts, "list tasks")?);
        }
        Ok(tasks)
    }

    fn get(&self, id: i64) -> TaskResult<Task> {
        let connection = self.lock("get task")?;
        query_task(&*connection, id, "get task")
    }

    fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task> {
        let mut connection = self.lock("update task")?;
        let transaction = connection
            .transaction()
            .map_err(|error| TaskError::storage("update task", error))?;
        // Read the current row first so an absent ID is a not-found error and an
        // omitted patch field keeps its stored value.
        let current = query_task(&transaction, id, "update task")?;
        let title = patch.title.as_deref().unwrap_or(current.title());
        let completed = patch.completed.unwrap_or(current.completed());
        // Re-validate the merged result before writing it back.
        Task::from_parts(id, title, completed)?;
        transaction
            .execute(
                "UPDATE tasks SET title = ?1, completed = ?2 WHERE id = ?3",
                params![title, bool_integer(completed), id],
            )
            .map_err(|error| TaskError::storage("update task", error))?;
        let updated = query_task(&transaction, id, "update task")?;
        transaction
            .commit()
            .map_err(|error| TaskError::storage("update task", error))?;
        Ok(updated)
    }

    fn delete(&self, id: i64) -> TaskResult<()> {
        let mut connection = self.lock("delete task")?;
        let transaction = connection
            .transaction()
            .map_err(|error| TaskError::storage("delete task", error))?;
        let affected = transaction
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])
            .map_err(|error| TaskError::storage("delete task", error))?;
        // Zero affected rows means the ID never existed; report it before
        // committing an empty transaction.
        if affected == 0 {
            return Err(TaskError::not_found(id));
        }
        transaction
            .commit()
            .map_err(|error| TaskError::storage("delete task", error))
    }
}

// Reading one task by ID is identical whether run on a bare connection or inside
// a transaction, so both share this small trait instead of duplicating the SQL.
trait QueryTask {
    fn task_parts(&self, id: i64) -> Result<Option<(i64, String, i64)>, SqliteError>;
}

impl QueryTask for Connection {
    fn task_parts(&self, id: i64) -> Result<Option<(i64, String, i64)>, SqliteError> {
        self.query_row(
            "SELECT id, title, completed FROM tasks WHERE id = ?1",
            params![id],
            read_parts,
        )
        .optional()
    }
}

impl QueryTask for Transaction<'_> {
    fn task_parts(&self, id: i64) -> Result<Option<(i64, String, i64)>, SqliteError> {
        self.query_row(
            "SELECT id, title, completed FROM tasks WHERE id = ?1",
            params![id],
            read_parts,
        )
        .optional()
    }
}

fn query_task(query: &impl QueryTask, id: i64, operation: &'static str) -> TaskResult<Task> {
    let parts = query
        .task_parts(id)
        .map_err(|error| TaskError::storage(operation, error))?
        .ok_or_else(|| TaskError::not_found(id))?;
    task_from_parts(parts, operation)
}

fn read_parts(row: &rusqlite::Row<'_>) -> rusqlite::Result<(i64, String, i64)> {
    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
}

// Maps a raw row into a validated `Task`. An out-of-range `completed` integer or
// an invalid stored title is treated as corruption (a storage error), not fixed.
fn task_from_parts(
    (id, title, completed): (i64, String, i64),
    operation: &'static str,
) -> TaskResult<Task> {
    let completed = match completed {
        0 => false,
        1 => true,
        value => {
            return Err(TaskError::storage(
                operation,
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid completed value {value} for task {id}"),
                ),
            ));
        }
    };
    Task::from_parts(id, title, completed).map_err(|error| TaskError::storage(operation, error))
}

// Creates the table if missing, then confirms the persisted schema matches the
// expected one so an incompatible database fails loudly instead of corrupting.
fn initialize_schema(connection: &Connection) -> TaskResult<()> {
    connection
        .execute_batch(INITIALIZE_SCHEMA)
        .map_err(|error| TaskError::storage("initialize sqlite schema", error))?;
    let statement: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'tasks'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| TaskError::storage("inspect sqlite schema", error))?;
    if canonical_sql(&statement) != canonical_sql(SCHEMA) {
        return Err(TaskError::storage(
            "inspect sqlite schema",
            io::Error::new(io::ErrorKind::InvalidData, "incompatible tasks schema"),
        ));
    }
    Ok(())
}

// Compares schema text ignoring whitespace, semicolons, and case, so cosmetic
// formatting differences do not read as an incompatible schema.
fn canonical_sql(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace() && *character != ';')
        .flat_map(char::to_lowercase)
        .collect()
}

fn bool_integer(value: bool) -> i64 {
    i64::from(value)
}

// Resolves `path` to an absolute path. An existing file is canonicalized; a
// not-yet-created file keeps its (canonicalized) parent so the store location is
// stable regardless of the process working directory.
fn absolute_target(path: &Path, operation: &'static str) -> TaskResult<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| TaskError::storage(operation, error))?
            .join(path)
    };
    if absolute.exists() {
        std::fs::canonicalize(&absolute).map_err(|error| TaskError::storage(operation, error))
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
            std::fs::canonicalize(parent).map_err(|error| TaskError::storage(operation, error))?;
        Ok(parent.join(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A panic while holding the connection lock must surface as a storage error
    // (tagged with the attempted operation), not a second panic.
    #[test]
    fn poisoned_lock_is_a_storage_error() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let repository =
            SqliteRepository::open(directory.path().join("tasks.db")).expect("open repository");
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = repository
                .connection
                .lock()
                .expect("lock SQLite connection");
            panic!("poison SQLite lock");
        }));
        let error = repository
            .list(TaskFilter::default())
            .expect_err("poisoned lock must fail");
        assert_eq!(error.storage_operation(), Some("list tasks"));
    }
}
