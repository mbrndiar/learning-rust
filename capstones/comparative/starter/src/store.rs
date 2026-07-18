//! Injectable persistence boundary for the comparative capstone.
//!
//! The application depends only on this trait, never on a concrete database, so a
//! real backing store and an in-memory test fake are interchangeable. Implementing a
//! store is where concurrency and durability decisions live; the four methods below
//! are the entire contract the application relies on.

use crate::{
    DeleteExpectation, DeleteResult, Entry, Key, KvError, ListResult, SetExpectation, SetResult,
};
use serde_json::Value;

/// Frozen SQLite busy timeout used by the shared observable error contract.
pub const BUSY_TIMEOUT_MS: u64 = 10_000;

/// Persistence operations required by the storage-independent application.
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
