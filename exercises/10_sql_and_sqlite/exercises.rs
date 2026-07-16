//! Exercises for module 10: a focused SQLite repository.

use rusqlite::Connection;

#[derive(Debug, PartialEq, Eq)]
pub struct Book {
    pub id: i64,
    pub author_id: i64,
    pub title: String,
    pub pages: i64,
}

pub fn create_schema(_connection: &Connection) -> rusqlite::Result<()> {
    todo!("enable foreign keys and create constrained authors and books tables")
}

pub fn add_author(_connection: &Connection, _name: &str) -> rusqlite::Result<i64> {
    todo!("insert with a bound parameter and return the generated id")
}

pub fn add_book(
    _connection: &Connection,
    _author_id: i64,
    _title: &str,
    _pages: i64,
) -> rusqlite::Result<i64> {
    todo!("insert a book with bound parameters and return the generated id")
}

pub fn find_book(_connection: &Connection, _id: i64) -> rusqlite::Result<Option<Book>> {
    todo!("map the selected row and convert QueryReturnedNoRows to None")
}

pub fn books_with_at_least(
    _connection: &Connection,
    _minimum_pages: i64,
) -> rusqlite::Result<Vec<Book>> {
    todo!("filter with a parameter and collect explicitly mapped rows")
}

pub fn author_page_totals(_connection: &Connection) -> rusqlite::Result<Vec<(String, i64)>> {
    todo!("join authors to books, aggregate pages, and order deterministically")
}

pub fn insert_then_rollback(_connection: &mut Connection, _author_id: i64) -> rusqlite::Result<()> {
    todo!("insert inside a transaction, then roll it back")
}

fn main() {
    println!("Run `cargo test --example ex-10-sqlite` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn database() -> (tempfile::TempDir, Connection) {
        let directory = tempdir().expect("temporary directory");
        let connection =
            Connection::open(directory.path().join("exercise.sqlite")).expect("database");
        create_schema(&connection).expect("schema");
        (directory, connection)
    }

    #[test]
    fn supports_parameterized_crud_mapping_and_filters() {
        let (_directory, connection) = database();
        let author_id = add_author(&connection, "Octavia Butler").expect("author");
        let short = add_book(&connection, author_id, "Kindred", 264).expect("book");
        let long = add_book(&connection, author_id, "Parable", 345).expect("book");

        assert_eq!(
            find_book(&connection, short).expect("query"),
            Some(Book {
                id: short,
                author_id,
                title: String::from("Kindred"),
                pages: 264,
            })
        );
        assert_eq!(find_book(&connection, 999).expect("query"), None);
        assert_eq!(
            books_with_at_least(&connection, 300).expect("filter"),
            vec![Book {
                id: long,
                author_id,
                title: String::from("Parable"),
                pages: 345,
            }]
        );
    }

    #[test]
    fn joins_aggregates_and_rolls_back() {
        let (_directory, mut connection) = database();
        let author_id = add_author(&connection, "Ursula Le Guin").expect("author");
        add_book(&connection, author_id, "Earthsea", 210).expect("book");
        add_book(&connection, author_id, "The Dispossessed", 341).expect("book");

        assert_eq!(
            author_page_totals(&connection).expect("aggregate"),
            vec![(String::from("Ursula Le Guin"), 551)]
        );
        insert_then_rollback(&mut connection, author_id).expect("rollback");
        assert_eq!(
            author_page_totals(&connection).expect("aggregate"),
            vec![(String::from("Ursula Le Guin"), 551)]
        );
    }
}
