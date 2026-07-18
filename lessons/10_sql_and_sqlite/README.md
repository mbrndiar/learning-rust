# 🗃️🦀 Module 10: SQL and SQLite

Relational databases store rows under a schema and let SQL describe which data
to read or change. This module teaches the portable relational ideas first, then
calls out SQLite-specific behavior explicitly. The examples use
[`rusqlite`](https://docs.rs/rusqlite/) and real database files inside temporary
directories; no service or separately installed SQLite library is required.

## 🎯 Learning objectives

After this module, you should be able to design tables with primary, foreign-key,
unique, check, and not-null constraints; own a connection as a resource; use
parameterized statements; implement CRUD with explicit row mapping and
`QueryReturnedNoRows`; write joins, aggregates, and indexes; use transactions and
generated IDs; explain SQLite affinity, locking, and pragmas; and place a narrow
repository trait in front of persistence.

## 🧱 Relational model and SQL

A table has named columns and constraints. A primary key identifies a row, while
a foreign key relates one table to another. Constraints make invalid states fail
at the database boundary. SQL is set-oriented: a query describes the result,
rather than prescribing a row-by-row algorithm.

Always bind values with placeholders such as `?1`. String-building SQL from input
mixes code with data, risks injection, and mishandles quoting. Table and column
names cannot normally be parameters; choose identifiers from trusted code.

Constraints make important invariants executable at the storage boundary:

```sql
CREATE TABLE books (
    id          INTEGER PRIMARY KEY,
    author_id   INTEGER NOT NULL REFERENCES authors(id),
    title       TEXT NOT NULL CHECK (length(trim(title)) > 0),
    pages       INTEGER NOT NULL CHECK (pages > 0)
);
```

Rust values are then bound separately from the SQL text. The runnable lesson
creates the connection and values surrounding this statement:

```rust
connection.execute(
    "INSERT INTO books (author_id, title, pages) VALUES (?1, ?2, ?3)",
    params![author_id, title, 42],
)?;
```

## 🪶 SQLite specifics

SQLite is an embedded database: opening a `rusqlite::Connection` owns a database
connection in the current process. SQLite uses type affinity rather than the
strict column typing of many server databases, serializes competing writes, and
configures connection behavior with pragmas. Enable foreign keys on every
connection that relies on them and set a finite busy timeout when lock contention
is possible.

This module intentionally omits ORMs, migration frameworks, production pool
tuning, and HTTP. Those are separate design topics.

`query_row` reports a missing row as `QueryReturnedNoRows`; `OptionalExtension`
can deliberately translate that expected absence into `Option`:

```rust
use rusqlite::OptionalExtension;

let book = find_book(&connection, id).optional()?;
```

The complete CRUD lesson defines `Book`, `find_book`, and the surrounding
fallible function.

## 📘 Lessons

- `01_schema_constraints_and_parameters.rs` — schema, constraints, ownership,
  parameterized inserts
- `02_crud_queries_and_indexes.rs` — CRUD, row mapping, absence, joins,
  aggregates, indexes
- `03_transactions_and_repositories.rs` — rollback, generated IDs, affinity,
  pragmas, locking, narrow traits

## 🚀 Running

```bash
cargo run --example lesson-10-schema-parameters
cargo run --example lesson-10-crud-queries
cargo run --example lesson-10-transactions-repositories
```

Then practice with
[`exercises/10_sql_and_sqlite/`](../../exercises/10_sql_and_sqlite/README.md).
Continue to [Module 11: Concurrency](../11_concurrency/README.md).

## 🚧 Common mistakes

- Formatting user values into SQL instead of binding parameters.
- Forgetting that `Connection`, statements, rows, and transactions own resources.
- Treating a missing row as a corrupt database instead of handling
  `QueryReturnedNoRows`.
- Assuming SQLite enforces foreign keys without enabling the pragma.
- Assuming a declared SQLite column type behaves like strict server-database
  typing.
- Holding a write transaction open while doing unrelated work.
- Adding an index without checking the queries and write cost it supports.

## 🧠 Review questions

1. Which invariants belong in database constraints?
2. Why are bound parameters safer and more correct than SQL string formatting?
3. How does explicit row mapping connect SQL columns to Rust types?
4. When should a missing row become `Option<T>`?
5. What does a transaction guarantee when one statement fails?
6. Which SQLite behaviors differ from a typical client/server database?
7. Why should a repository trait expose domain operations rather than SQL details?
