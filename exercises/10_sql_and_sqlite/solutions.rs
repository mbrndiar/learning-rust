//! Reference solution for module 10.

use rusqlite::{Connection, OptionalExtension, params};
use tempfile::tempdir;

#[derive(Debug, PartialEq, Eq)]
struct Book {
    id: i64,
    author_id: i64,
    title: String,
    pages: i64,
}

fn create_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE authors (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE CHECK (length(trim(name)) > 0)
        );
        CREATE TABLE books (
            id INTEGER PRIMARY KEY,
            author_id INTEGER NOT NULL REFERENCES authors(id),
            title TEXT NOT NULL CHECK (length(trim(title)) > 0),
            pages INTEGER NOT NULL CHECK (pages > 0)
        );
        CREATE INDEX books_pages ON books(pages);
        ",
    )
}

fn add_author(connection: &Connection, name: &str) -> rusqlite::Result<i64> {
    connection.execute("INSERT INTO authors (name) VALUES (?1)", [name])?;
    Ok(connection.last_insert_rowid())
}

fn add_book(
    connection: &Connection,
    author_id: i64,
    title: &str,
    pages: i64,
) -> rusqlite::Result<i64> {
    connection.execute(
        "INSERT INTO books (author_id, title, pages) VALUES (?1, ?2, ?3)",
        params![author_id, title, pages],
    )?;
    Ok(connection.last_insert_rowid())
}

fn map_book(row: &rusqlite::Row<'_>) -> rusqlite::Result<Book> {
    Ok(Book {
        id: row.get(0)?,
        author_id: row.get(1)?,
        title: row.get(2)?,
        pages: row.get(3)?,
    })
}

fn find_book(connection: &Connection, id: i64) -> rusqlite::Result<Option<Book>> {
    connection
        .query_row(
            "SELECT id, author_id, title, pages FROM books WHERE id = ?1",
            [id],
            map_book,
        )
        .optional()
}

fn books_with_at_least(connection: &Connection, minimum_pages: i64) -> rusqlite::Result<Vec<Book>> {
    let mut statement = connection.prepare(
        "SELECT id, author_id, title, pages
         FROM books WHERE pages >= ?1 ORDER BY pages, id",
    )?;
    statement
        .query_map([minimum_pages], map_book)?
        .collect::<rusqlite::Result<Vec<_>>>()
}

fn author_page_totals(connection: &Connection) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut statement = connection.prepare(
        "SELECT authors.name, COALESCE(SUM(books.pages), 0)
         FROM authors
         LEFT JOIN books ON books.author_id = authors.id
         GROUP BY authors.id, authors.name
         ORDER BY authors.name",
    )?;
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()
}

fn insert_then_rollback(connection: &mut Connection, author_id: i64) -> rusqlite::Result<()> {
    let transaction = connection.transaction()?;
    transaction.execute(
        "INSERT INTO books (author_id, title, pages) VALUES (?1, ?2, ?3)",
        params![author_id, "Rolled back", 1],
    )?;
    transaction.rollback()
}

fn main() -> rusqlite::Result<()> {
    let directory = tempdir().expect("temporary directory");
    let mut connection =
        Connection::open(directory.path().join("solution.sqlite")).expect("database");
    create_schema(&connection)?;
    let author_id = add_author(&connection, "Octavia Butler")?;
    let book_id = add_book(&connection, author_id, "Kindred", 264)?;

    assert_eq!(find_book(&connection, book_id)?.expect("book").pages, 264);
    assert_eq!(books_with_at_least(&connection, 200)?.len(), 1);
    assert_eq!(
        author_page_totals(&connection)?,
        vec![(String::from("Octavia Butler"), 264)]
    );
    insert_then_rollback(&mut connection, author_id)?;
    assert_eq!(books_with_at_least(&connection, 0)?.len(), 1);
    println!("Module 10 SQLite solution passed.");
    Ok(())
}
