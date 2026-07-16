# 🗃️🧩 Exercises: Module 10 — SQL and SQLite

Complete a small library repository backed by a real temporary SQLite database:

- create constrained author and book tables;
- bind parameters for inserts and filters;
- map rows explicitly and return `None` for a missing row;
- filter books by a minimum page count;
- produce one author/book aggregate through a join; and
- prove a transaction rollback leaves no inserted row.

Run:

```bash
cargo test --example ex-10-sqlite
cargo run --example solution-10-sqlite
```

The exercise deliberately excludes HTTP, ORMs, migration frameworks, and
connection-pool tuning.

## 💡 Hint ladder

1. Enable foreign keys before creating or changing related rows.
2. Use `?1`, `?2`, and `params!`; never interpolate values into SQL.
3. Match `Error::QueryReturnedNoRows`, or import `OptionalExtension` and use
   `query_row(...).optional()`, for expected absence.
4. Keep `row.get` positions aligned with the `SELECT` list.
5. Call `rollback()` explicitly in the rollback function.
