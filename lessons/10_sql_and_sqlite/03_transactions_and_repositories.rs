//! Lesson 10.3: transactions, SQLite behavior, and a narrow repository trait.

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::error::Error;
use std::time::Duration;
use tempfile::tempdir;

#[derive(Debug, PartialEq, Eq)]
struct Note {
    id: i64,
    body: String,
}

trait NoteRepository {
    fn add(&mut self, body: &str) -> rusqlite::Result<i64>;
    fn find(&self, id: i64) -> rusqlite::Result<Option<Note>>;
}

struct SqliteNotes {
    connection: Connection,
}

impl SqliteNotes {
    fn open(path: &std::path::Path) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_millis(250))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.execute(
            "CREATE TABLE notes (
                id INTEGER PRIMARY KEY,
                body TEXT NOT NULL CHECK (length(trim(body)) > 0)
            ) STRICT",
            [],
        )?;
        Ok(Self { connection })
    }
}

impl NoteRepository for SqliteNotes {
    fn add(&mut self, body: &str) -> rusqlite::Result<i64> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute("INSERT INTO notes (body) VALUES (?1)", [body])?;
        let id = transaction.last_insert_rowid();
        transaction.commit()?;
        Ok(id)
    }

    fn find(&self, id: i64) -> rusqlite::Result<Option<Note>> {
        self.connection
            .query_row("SELECT id, body FROM notes WHERE id = ?1", [id], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    body: row.get(1)?,
                })
            })
            .optional()
    }
}

fn demonstrate_rollback(connection: &mut Connection) -> rusqlite::Result<()> {
    let transaction = connection.transaction()?;
    transaction.execute("INSERT INTO notes (body) VALUES (?1)", params!["temporary"])?;
    transaction.rollback()
}

fn main() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let mut repository = SqliteNotes::open(&directory.path().join("notes.sqlite"))?;

    let id = repository.add("Learn transaction boundaries")?;
    println!("inserted={:?}", repository.find(id)?);

    demonstrate_rollback(&mut repository.connection)?;
    let count: i64 = repository
        .connection
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
    assert_eq!(count, 1);

    // SQLite normally uses affinity; STRICT tables opt this table into stronger
    // type checks. Competing writers are serialized, so keep transactions short.
    let journal_mode: String =
        repository
            .connection
            .pragma_query_value(None, "journal_mode", |row| row.get(0))?;
    println!("journal_mode={journal_mode}");
    Ok(())
}
