//! SQLite persistence for the comparative capstone.
//!
//! A single [`Connection`] backs one store. Writes run inside `BEGIN IMMEDIATE`
//! transactions so the reserved write lock is taken up front — this both serializes
//! concurrent writers (via the busy timeout) and gives read-modify-write sequences a
//! stable snapshot for optimistic-concurrency (`Absent`/`Exact`) checks.
//!
//! On open the schema is classified by comparing each object's *canonical* SQL
//! (whitespace/quotes/case stripped) against known definitions: an empty database is
//! initialized to v1, an exact v0 database is migrated in place, and an exact v1
//! database is revalidated. Anything else is rejected as malformed rather than
//! silently coerced. Migration assigns revisions `1..=N` to existing rows in `BINARY`
//! key order, which is also the canonical order used by `list`.

use crate::domain::{MAX_SAFE_INTEGER, parse_stored_json};
use crate::{
    DeleteExpectation, DeleteResult, Entry, Key, KvError, ListResult, Revision, SetExpectation,
    SetResult,
};
use rusqlite::{
    Connection, Error as SqlError, ErrorCode, OpenFlags, OptionalExtension, Transaction,
    TransactionBehavior, params,
};
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

/// How long a blocked operation waits for the write lock before returning `Busy`.
pub const BUSY_TIMEOUT_MS: u64 = 10_000;

// v1 schema DDL. The CHECK constraints encode the store invariants directly in
// SQLite so a foreign writer cannot leave the database in an out-of-range state.
const CREATE_METADATA: &str = "
    CREATE TABLE store_metadata (
        singleton       INTEGER PRIMARY KEY CHECK (singleton = 1),
        schema_version  INTEGER NOT NULL CHECK (schema_version = 1),
        global_revision INTEGER NOT NULL
                        CHECK (global_revision BETWEEN 0 AND 9007199254740991)
    )";
const CREATE_ENTRIES: &str = "
    CREATE TABLE entries (
        key        TEXT PRIMARY KEY COLLATE BINARY,
        value_json TEXT NOT NULL CHECK (json_valid(value_json)),
        revision   INTEGER NOT NULL
                   CHECK (revision BETWEEN 1 AND 9007199254740991)
    )";
const INSERT_METADATA: &str = "
    INSERT INTO store_metadata(singleton, schema_version, global_revision)
    VALUES (1, 1, 0)";

// Canonical (whitespace/quote/case-stripped) forms of the recognized schemas, used
// by `canonical_sql` comparisons to detect exact v0 and v1 databases.
const V0_ENTRIES_CANONICAL: &str =
    "createtableentries(keytextprimarykeycollatebinary,value_jsontextnotnull)";
const V1_ENTRIES_CANONICAL: &str = "createtableentries(keytextprimarykeycollatebinary,value_jsontextnotnullcheck(json_valid(value_json)),revisionintegernotnullcheck(revisionbetween1and9007199254740991))";
const V1_METADATA_CANONICAL: &str = "createtablestore_metadata(singletonintegerprimarykeycheck(singleton=1),schema_versionintegernotnullcheck(schema_version=1),global_revisionintegernotnullcheck(global_revisionbetween0and9007199254740991))";

/// Persistence operations required by the storage-independent application.
///
/// Implementors own concurrency control and persistence behavior; the application
/// layer only sees these four typed operations and never touches SQL directly.
pub trait KvStore {
    /// Writes `value` at `key` subject to `expectation`, returning the new revision.
    fn set(
        &mut self,
        key: &Key,
        value: &Value,
        expectation: SetExpectation,
    ) -> Result<SetResult, KvError>;
    /// Reads the current entry for `key`, or `NotFound` if absent.
    fn get(&self, key: &Key) -> Result<Entry, KvError>;
    /// Removes `key` subject to `expectation`, returning the removed revision.
    fn delete(
        &mut self,
        key: &Key,
        expectation: DeleteExpectation,
    ) -> Result<DeleteResult, KvError>;
    /// Lists all live entries in canonical (`BINARY` key) order.
    fn list(&self) -> Result<ListResult, KvError>;
}

/// One configured SQLite connection implementing the v1 store.
pub struct SqliteStore {
    connection: Connection,
}

impl SqliteStore {
    /// Opens, configures, initializes, migrates, and validates a database.
    pub fn open(path: &Path) -> Result<Self, KvError> {
        // NO_MUTEX: this connection is used from a single thread, so SQLite's own
        // per-connection mutex is unnecessary.
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let mut connection =
            Connection::open_with_flags(path, flags).map_err(|error| map_sql(error, "open"))?;
        // Block (rather than fail immediately) when another writer holds the lock.
        connection
            .busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))
            .map_err(|error| map_sql(error, "configure"))?;
        configure_journal_mode(&connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(|error| map_sql(error, "configure"))?;

        prepare_schema(&mut connection)?;
        Ok(Self { connection })
    }
}

fn configure_journal_mode(connection: &Connection) -> Result<(), KvError> {
    // Switching journal modes needs the database lock; retry on transient busy
    // errors until the busy-timeout deadline, then surface the failure.
    let deadline = Instant::now() + Duration::from_millis(BUSY_TIMEOUT_MS);
    loop {
        match connection.query_row("PRAGMA journal_mode=WAL", [], |row| row.get::<_, String>(0)) {
            Ok(mode) if mode.eq_ignore_ascii_case("wal") => return Ok(()),
            // The pragma succeeded but WAL was refused (e.g. unsupported VFS).
            Ok(_) => {
                return Err(KvError::Storage {
                    operation: "configure",
                });
            }
            Err(error) if is_busy_error(&error) && Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(map_sql(error, "configure")),
        }
    }
}

impl KvStore for SqliteStore {
    fn set(
        &mut self,
        key: &Key,
        value: &Value,
        expectation: SetExpectation,
    ) -> Result<SetResult, KvError> {
        // IMMEDIATE takes the write lock before reading, so the current revision we
        // observe cannot change under us before we commit the conditional write.
        let transaction = begin_immediate(&mut self.connection, "write")?;
        let current_revision = query_entry_revision(&transaction, key, "write")?;
        match expectation {
            SetExpectation::Any => {}
            // Absent: creation only; an existing key is a conflict.
            SetExpectation::Absent if current_revision.is_some() => {
                return Err(KvError::ConflictAbsent {
                    key: key.as_str().to_owned(),
                    actual: current_revision.expect("checked as present"),
                });
            }
            SetExpectation::Absent => {}
            // Exact: compare-and-set; the stored revision must match exactly.
            SetExpectation::Exact(expected) if current_revision != Some(expected.get()) => {
                return Err(KvError::ConflictExact {
                    key: key.as_str().to_owned(),
                    expected: expected.get(),
                    actual: current_revision,
                });
            }
            SetExpectation::Exact(_) => {}
        }

        let next_revision = next_revision(&transaction)?;
        let value_json =
            serde_json::to_string(value).map_err(|_| KvError::Storage { operation: "write" })?;
        transaction
            .execute(
                "INSERT INTO entries(key, value_json, revision) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE
                 SET value_json = excluded.value_json, revision = excluded.revision",
                params![key.as_str(), value_json, next_revision],
            )
            .map_err(|error| map_sql(error, "write"))?;
        update_global_revision(&transaction, next_revision)?;
        transaction
            .commit()
            .map_err(|error| map_sql(error, "commit"))?;

        Ok(SetResult {
            entry: Entry {
                key: key.clone(),
                value: value.clone(),
                revision: revision_from_store(next_revision)?,
            },
            created: current_revision.is_none(),
        })
    }

    fn get(&self, key: &Key) -> Result<Entry, KvError> {
        let row = self
            .connection
            .query_row(
                "SELECT value_json, revision FROM entries WHERE key = ?1",
                [key.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(|error| map_sql(error, "read"))?;
        let Some((value_json, revision)) = row else {
            return Err(KvError::NotFound {
                key: key.as_str().to_owned(),
            });
        };
        let value = parse_stored_json(&value_json).map_err(|_| KvError::InvalidStorage {
            reason: "invalid_value",
            key: Some(key.as_str().to_owned()),
        })?;
        Ok(Entry {
            key: key.clone(),
            value,
            revision: revision_from_store(revision)?,
        })
    }

    fn delete(
        &mut self,
        key: &Key,
        expectation: DeleteExpectation,
    ) -> Result<DeleteResult, KvError> {
        let transaction = begin_immediate(&mut self.connection, "write")?;
        let current_revision =
            query_entry_revision(&transaction, key, "write")?.ok_or_else(|| KvError::NotFound {
                key: key.as_str().to_owned(),
            })?;
        if let DeleteExpectation::Exact(expected) = expectation {
            if current_revision != expected.get() {
                return Err(KvError::ConflictExact {
                    key: key.as_str().to_owned(),
                    expected: expected.get(),
                    actual: Some(current_revision),
                });
            }
        }

        let next_revision = next_revision(&transaction)?;
        transaction
            .execute("DELETE FROM entries WHERE key = ?1", [key.as_str()])
            .map_err(|error| map_sql(error, "write"))?;
        update_global_revision(&transaction, next_revision)?;
        transaction
            .commit()
            .map_err(|error| map_sql(error, "commit"))?;
        Ok(DeleteResult {
            deleted_revision: Revision::new(current_revision).map_err(|_| revision_invariant())?,
            revision: revision_from_store(next_revision)?,
        })
    }

    fn list(&self) -> Result<ListResult, KvError> {
        // LEFT JOIN ON TRUE against the singleton metadata row guarantees at least
        // one result row even when `entries` is empty, so `global_revision` is
        // always returned; entry columns are NULL in that empty case.
        let mut statement = self
            .connection
            .prepare(
                "SELECT entries.key, entries.value_json, entries.revision,
                        store_metadata.global_revision
                 FROM store_metadata
                 LEFT JOIN entries ON TRUE
                 WHERE store_metadata.singleton = 1
                 ORDER BY entries.key COLLATE BINARY",
            )
            .map_err(|error| map_sql(error, "read"))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|error| map_sql(error, "read"))?;
        let mut entries = Vec::new();
        let mut global_revision = None;
        for row in rows {
            let (key_text, value_json, revision, row_global_revision) =
                row.map_err(|error| map_sql(error, "read"))?;
            global_revision = Some(row_global_revision);
            // The empty-table row has a NULL key; skip it but keep global_revision.
            let Some(key_text) = key_text else {
                continue;
            };
            let value_json = value_json.ok_or_else(malformed_schema)?;
            let revision = revision.ok_or_else(malformed_schema)?;
            let key = Key::parse(&key_text).map_err(|_| KvError::InvalidStorage {
                reason: "invalid_key",
                key: Some(key_text.clone()),
            })?;
            let value = parse_stored_json(&value_json).map_err(|_| KvError::InvalidStorage {
                reason: "invalid_value",
                key: Some(key_text),
            })?;
            entries.push(Entry {
                key,
                value,
                revision: revision_from_store(revision)?,
            });
        }
        Ok(ListResult {
            entries,
            global_revision: u64::try_from(global_revision.ok_or_else(malformed_schema)?).map_err(
                |_| KvError::InvalidStorage {
                    reason: "revision_invariant",
                    key: None,
                },
            )?,
        })
    }
}

fn prepare_schema(connection: &mut Connection) -> Result<(), KvError> {
    let transaction = begin_immediate(connection, "initialize")?;
    // Refuse databases written by a newer schema version outright.
    if let Some(found) = future_schema_version(&transaction).filter(|version| *version > 1) {
        return Err(KvError::UnsupportedSchema { found });
    }
    ensure_integrity(&transaction)?;
    let objects = application_objects(&transaction)?;

    // Classify by exact schema shape: empty -> create v1; exact v0 -> migrate;
    // exact v1 -> revalidate; anything else is malformed and never coerced.
    if objects.is_empty() {
        validate_default_pragmas(&transaction)?;
        initialize(&transaction)?;
    } else if is_exact_v0(&objects) {
        validate_default_pragmas(&transaction)?;
        migrate_v0(&transaction)?;
    } else if is_exact_v1(&objects) {
        validate_default_pragmas(&transaction)?;
        validate_v1(&transaction)?;
    } else {
        return Err(KvError::InvalidStorage {
            reason: "malformed_schema",
            key: None,
        });
    }

    // Our databases never set these pragmas; a non-zero value signals a foreign
    // database that merely happens to share our table shape.
    fn validate_default_pragmas(transaction: &Transaction<'_>) -> Result<(), KvError> {
        let user_version = transaction
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .map_err(|_| malformed_schema())?;
        let application_id = transaction
            .query_row("PRAGMA application_id", [], |row| row.get::<_, i64>(0))
            .map_err(|_| malformed_schema())?;
        if user_version == 0 && application_id == 0 {
            Ok(())
        } else {
            Err(malformed_schema())
        }
    }

    transaction
        .commit()
        .map_err(|error| map_sql(error, "commit"))
}

fn initialize(transaction: &Transaction<'_>) -> Result<(), KvError> {
    transaction
        .execute_batch(&format!(
            "{CREATE_METADATA};{CREATE_ENTRIES};{INSERT_METADATA};"
        ))
        .map_err(|error| map_sql(error, "initialize"))
}

fn migrate_v0(transaction: &Transaction<'_>) -> Result<(), KvError> {
    // Read every legacy row in canonical BINARY key order; that order determines
    // the revisions assigned below, so the migration is deterministic.
    let rows = {
        let mut statement = transaction
            .prepare(
                "SELECT key, value_json
                 FROM entries
                 ORDER BY key COLLATE BINARY",
            )
            .map_err(|error| map_sql(error, "migrate"))?;
        let mapped = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| map_sql(error, "migrate"))?;
        let mut rows = Vec::new();
        for row in mapped {
            rows.push(row.map_err(|error| map_sql(error, "migrate"))?);
        }
        rows
    };

    // Validate keys and re-serialize values into canonical form before re-insert;
    // legacy data that fails validation aborts the migration.
    let mut normalized = Vec::with_capacity(rows.len());
    for (key_text, value_json) in rows {
        Key::parse(&key_text).map_err(|_| KvError::InvalidStorage {
            reason: "invalid_key",
            key: Some(key_text.clone()),
        })?;
        let value = crate::parse_json_value(&value_json).map_err(|_| KvError::InvalidStorage {
            reason: "invalid_value",
            key: Some(key_text.clone()),
        })?;
        normalized.push((
            key_text,
            serde_json::to_string(&value).map_err(|_| KvError::Storage {
                operation: "migrate",
            })?,
        ));
    }

    // Rebuild v1 tables alongside the renamed legacy table, then repopulate.
    transaction
        .execute_batch(&format!(
            "ALTER TABLE entries RENAME TO entries_v0_migration;
             {CREATE_METADATA};
             {CREATE_ENTRIES};
             {INSERT_METADATA};"
        ))
        .map_err(|error| map_sql(error, "migrate"))?;
    for (index, (key, value_json)) in normalized.iter().enumerate() {
        // Assign revisions 1..=N in the canonical key order established above.
        let revision = i64::try_from(index + 1).map_err(|_| KvError::RevisionExhausted)?;
        transaction
            .execute(
                "INSERT INTO entries(key, value_json, revision) VALUES (?1, ?2, ?3)",
                params![key, value_json, revision],
            )
            .map_err(|error| map_sql(error, "migrate"))?;
    }
    // The global revision equals the number of migrated rows.
    let global_revision =
        i64::try_from(normalized.len()).map_err(|_| KvError::RevisionExhausted)?;
    transaction
        .execute(
            "UPDATE store_metadata SET global_revision = ?1 WHERE singleton = 1",
            [global_revision],
        )
        .map_err(|error| map_sql(error, "migrate"))?;
    transaction
        .execute("DROP TABLE entries_v0_migration", [])
        .map_err(|error| map_sql(error, "migrate"))?;
    Ok(())
}

fn validate_v1(transaction: &Transaction<'_>) -> Result<(), KvError> {
    let metadata_rows = {
        let mut statement = transaction
            .prepare(
                "SELECT singleton, schema_version, global_revision
                 FROM store_metadata",
            )
            .map_err(|_| malformed_schema())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|_| malformed_schema())?;
        let mut values = Vec::new();
        for row in rows {
            values.push(row.map_err(|_| malformed_schema())?);
        }
        values
    };
    if metadata_rows.len() != 1 {
        return Err(malformed_schema());
    }
    let (singleton, schema_version, global_revision) = metadata_rows[0];
    if singleton != 1 || schema_version != 1 {
        return Err(malformed_schema());
    }
    if !(0..=i64::try_from(MAX_SAFE_INTEGER).expect("safe integer fits i64"))
        .contains(&global_revision)
    {
        return Err(revision_invariant());
    }

    // Every entry must parse, sit within `1..=global_revision`, and hold a unique
    // revision; a duplicate or out-of-range revision means a corrupt invariant.
    let mut seen_revisions = HashSet::new();
    let mut statement = transaction
        .prepare("SELECT key, value_json, revision FROM entries ORDER BY key COLLATE BINARY")
        .map_err(|_| malformed_schema())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|_| malformed_schema())?;
    for row in rows {
        let (key, value_json, revision) = row.map_err(|_| malformed_schema())?;
        Key::parse(&key).map_err(|_| KvError::InvalidStorage {
            reason: "invalid_key",
            key: Some(key.clone()),
        })?;
        parse_stored_json(&value_json).map_err(|_| KvError::InvalidStorage {
            reason: "invalid_value",
            key: Some(key),
        })?;
        if !(1..=global_revision).contains(&revision) || !seen_revisions.insert(revision) {
            return Err(revision_invariant());
        }
    }
    Ok(())
}

fn ensure_integrity(transaction: &Transaction<'_>) -> Result<(), KvError> {
    // `PRAGMA integrity_check` returns the single row "ok" on a healthy database;
    // any other output (or failure) is treated as an integrity failure.
    let mut statement = transaction
        .prepare("PRAGMA integrity_check")
        .map_err(|_| integrity_failure())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|_| integrity_failure())?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|_| integrity_failure())?);
    }
    if messages == ["ok"] {
        Ok(())
    } else {
        Err(integrity_failure())
    }
}

#[derive(Debug)]
struct SchemaObject {
    object_type: String,
    name: String,
    sql: Option<String>,
}

fn application_objects(transaction: &Transaction<'_>) -> Result<Vec<SchemaObject>, KvError> {
    let mut statement = transaction
        .prepare(
            "SELECT type, name, sql
             FROM sqlite_schema
             WHERE name NOT LIKE 'sqlite_%'
             ORDER BY type COLLATE BINARY, name COLLATE BINARY",
        )
        .map_err(|error| map_sql(error, "read"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(SchemaObject {
                object_type: row.get(0)?,
                name: row.get(1)?,
                sql: row.get(2)?,
            })
        })
        .map_err(|error| map_sql(error, "read"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| map_sql(error, "read"))
}

fn is_exact_v0(objects: &[SchemaObject]) -> bool {
    objects.len() == 1
        && objects[0].object_type == "table"
        && objects[0].name == "entries"
        && objects[0].sql.as_deref().map(canonical_sql).as_deref() == Some(V0_ENTRIES_CANONICAL)
}

fn is_exact_v1(objects: &[SchemaObject]) -> bool {
    if objects.len() != 2 {
        return false;
    }
    let entries = objects.iter().find(|object| object.name == "entries");
    let metadata = objects
        .iter()
        .find(|object| object.name == "store_metadata");
    matches!(entries, Some(object) if object.object_type == "table"
        && object.sql.as_deref().map(canonical_sql).as_deref() == Some(V1_ENTRIES_CANONICAL))
        && matches!(metadata, Some(object) if object.object_type == "table"
            && object.sql.as_deref().map(canonical_sql).as_deref()
                == Some(V1_METADATA_CANONICAL))
}

fn future_schema_version(transaction: &Transaction<'_>) -> Option<i64> {
    transaction
        .query_row(
            "SELECT schema_version FROM store_metadata LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok()
}

fn canonical_sql(sql: &str) -> String {
    // Normalize DDL for comparison: drop whitespace and every quoting style SQLite
    // may echo back (", ', `, [ ]), then lowercase, leaving only structural tokens.
    sql.chars()
        .filter(|character| {
            !character.is_ascii_whitespace() && !matches!(character, '"' | '\'' | '`' | '[' | ']')
        })
        .flat_map(char::to_lowercase)
        .collect()
}

fn begin_immediate<'a>(
    connection: &'a mut Connection,
    operation: &'static str,
) -> Result<Transaction<'a>, KvError> {
    // IMMEDIATE acquires the write lock at BEGIN rather than on first write, so a
    // read-then-write sequence cannot deadlock while upgrading a shared lock.
    connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| map_sql(error, operation))
}

fn query_entry_revision(
    transaction: &Transaction<'_>,
    key: &Key,
    operation: &'static str,
) -> Result<Option<u64>, KvError> {
    let revision = transaction
        .query_row(
            "SELECT revision FROM entries WHERE key = ?1",
            [key.as_str()],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| map_sql(error, operation))?;
    revision
        .map(|value| {
            u64::try_from(value).map_err(|_| KvError::InvalidStorage {
                reason: "revision_invariant",
                key: None,
            })
        })
        .transpose()
}

fn next_revision(transaction: &Transaction<'_>) -> Result<i64, KvError> {
    let current = transaction
        .query_row(
            "SELECT global_revision FROM store_metadata WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| map_sql(error, "write"))?;
    // Revisions are capped at MAX_SAFE_INTEGER so they round-trip exactly through
    // JSON's binary64 numbers; refuse to advance past that ceiling.
    if current == i64::try_from(MAX_SAFE_INTEGER).expect("safe integer fits i64") {
        return Err(KvError::RevisionExhausted);
    }
    current.checked_add(1).ok_or(KvError::RevisionExhausted)
}

fn update_global_revision(transaction: &Transaction<'_>, revision: i64) -> Result<(), KvError> {
    let updated = transaction
        .execute(
            "UPDATE store_metadata SET global_revision = ?1 WHERE singleton = 1",
            [revision],
        )
        .map_err(|error| map_sql(error, "write"))?;
    if updated == 1 {
        Ok(())
    } else {
        Err(malformed_schema())
    }
}

fn revision_from_store(value: i64) -> Result<Revision, KvError> {
    let value = u64::try_from(value).map_err(|_| revision_invariant())?;
    Revision::new(value).map_err(|_| revision_invariant())
}

fn map_sql(error: SqlError, operation: &'static str) -> KvError {
    // Translate SQLite failures into the store's taxonomy: lock contention becomes
    // Busy, corruption becomes an integrity failure, everything else is Storage.
    match &error {
        _ if is_busy_error(&error) => KvError::Busy,
        SqlError::SqliteFailure(failure, _)
            if matches!(
                failure.code,
                ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase
            ) =>
        {
            integrity_failure()
        }
        _ => KvError::Storage { operation },
    }
}

fn is_busy_error(error: &SqlError) -> bool {
    matches!(
        error,
        SqlError::SqliteFailure(failure, _)
            if matches!(
                failure.code,
                ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
            )
    )
}

fn malformed_schema() -> KvError {
    KvError::InvalidStorage {
        reason: "malformed_schema",
        key: None,
    }
}

fn revision_invariant() -> KvError {
    KvError::InvalidStorage {
        reason: "revision_invariant",
        key: None,
    }
}

fn integrity_failure() -> KvError {
    KvError::InvalidStorage {
        reason: "integrity_check_failed",
        key: None,
    }
}
