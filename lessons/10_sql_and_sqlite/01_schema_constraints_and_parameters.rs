//! Lesson 10.1: relational schema, connection ownership, and parameters.

use rusqlite::{Connection, params};
use std::error::Error;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let database_path = directory.path().join("library.sqlite");
    let connection = Connection::open(&database_path)?;

    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE authors (
            id      INTEGER PRIMARY KEY,
            name    TEXT NOT NULL UNIQUE CHECK (length(trim(name)) > 0)
        );

        CREATE TABLE books (
            id          INTEGER PRIMARY KEY,
            author_id   INTEGER NOT NULL REFERENCES authors(id),
            title       TEXT NOT NULL CHECK (length(trim(title)) > 0),
            pages       INTEGER NOT NULL CHECK (pages > 0),
            UNIQUE (author_id, title)
        );
        ",
    )?;

    let author = "Ada Lovelace";
    connection.execute("INSERT INTO authors (name) VALUES (?1)", [author])?;
    let author_id = connection.last_insert_rowid();

    let title = "Notes on the Analytical Engine";
    connection.execute(
        "INSERT INTO books (author_id, title, pages) VALUES (?1, ?2, ?3)",
        params![author_id, title, 42],
    )?;

    let duplicate = connection.execute("INSERT INTO authors (name) VALUES (?1)", [author]);
    assert!(
        duplicate.is_err(),
        "the UNIQUE constraint rejects duplicates"
    );

    let count: i64 = connection.query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))?;
    println!("database={} books={count}", database_path.display());

    // `connection` closes before `directory` removes the temporary database.
    Ok(())
}
