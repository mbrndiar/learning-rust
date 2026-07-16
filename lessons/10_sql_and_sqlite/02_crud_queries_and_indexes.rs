//! Lesson 10.2: CRUD, row mapping, joins, aggregates, and indexes.

use rusqlite::{Connection, Error as SqlError, OptionalExtension, params};
use std::error::Error;
use tempfile::tempdir;

#[derive(Debug, PartialEq, Eq)]
struct Book {
    id: i64,
    title: String,
    pages: i64,
}

fn find_book(connection: &Connection, id: i64) -> rusqlite::Result<Book> {
    connection.query_row(
        "SELECT id, title, pages FROM books WHERE id = ?1",
        [id],
        |row| {
            Ok(Book {
                id: row.get(0)?,
                title: row.get(1)?,
                pages: row.get(2)?,
            })
        },
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let connection = Connection::open(directory.path().join("books.sqlite"))?;
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE authors (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );
        CREATE TABLE books (
            id INTEGER PRIMARY KEY,
            author_id INTEGER NOT NULL REFERENCES authors(id),
            title TEXT NOT NULL,
            pages INTEGER NOT NULL CHECK (pages > 0)
        );
        CREATE INDEX books_author_pages ON books(author_id, pages);
        INSERT INTO authors (name) VALUES ('Ursula Le Guin'), ('Octavia Butler');
        ",
    )?;

    connection.execute(
        "INSERT INTO books (author_id, title, pages) VALUES (?1, ?2, ?3)",
        params![1, "A Wizard of Earthsea", 205],
    )?;
    let book_id = connection.last_insert_rowid();
    connection.execute(
        "UPDATE books SET pages = ?1 WHERE id = ?2",
        params![210, book_id],
    )?;

    assert_eq!(
        find_book(&connection, book_id)?,
        Book {
            id: book_id,
            title: String::from("A Wizard of Earthsea"),
            pages: 210,
        }
    );
    assert!(matches!(
        find_book(&connection, 999),
        Err(SqlError::QueryReturnedNoRows)
    ));

    let optional = find_book(&connection, 999).optional()?;
    assert_eq!(optional, None);

    let mut statement = connection.prepare(
        "
        SELECT authors.name, COUNT(books.id), COALESCE(SUM(books.pages), 0)
        FROM authors
        LEFT JOIN books ON books.author_id = authors.id
        GROUP BY authors.id, authors.name
        ORDER BY authors.name
        ",
    )?;
    let summaries = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    for summary in summaries {
        println!("{:?}", summary?);
    }

    let deleted = connection.execute("DELETE FROM books WHERE id = ?1", [book_id])?;
    assert_eq!(deleted, 1);
    Ok(())
}
