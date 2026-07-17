//! Storage-independent values for the shared key/value contract.
//!
//! Values on the wire are a *restricted* JSON: objects, arrays, strings, booleans,
//! null, and integers only — no floating point survives. This module defines the
//! validated key/revision types and the `Command`/result shapes; parsing and
//! canonical normalization of values is a milestone you implement here.

use crate::KvError;
use serde_json::Value;

/// Largest integer representable exactly in IEEE-754 binary64 (2^53 − 1).
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
/// Maximum byte length of a serialized value argument.
pub const MAX_VALUE_BYTES: usize = 65_536;
/// Maximum nesting depth for arrays and objects.
pub const MAX_CONTAINER_DEPTH: usize = 32;

/// A validated key from the shared ASCII key grammar.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(String);

impl Key {
    /// Milestone 1 TODO: parse and validate a key.
    pub fn parse(_value: &str) -> Result<Self, KvError> {
        Err(KvError::incomplete("key validation"))
    }

    /// Borrows the validated key text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A positive JSON-safe global revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(u64);

impl Revision {
    /// Milestone 1 TODO: validate a positive safe revision.
    pub fn new(_value: u64) -> Result<Self, KvError> {
        Err(KvError::incomplete("revision validation"))
    }

    /// Returns the numeric revision.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Conditional behavior supported by `set`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetExpectation {
    /// Write unconditionally.
    Any,
    /// Only create; fail with a conflict if the key already exists.
    Absent,
    /// Only overwrite when the current revision equals this one.
    Exact(Revision),
}

/// Conditional behavior supported by `delete`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteExpectation {
    /// Delete unconditionally (still fails if the key is absent).
    Any,
    /// Only delete when the current revision equals this one.
    Exact(Revision),
}

/// One live key/value entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The entry's validated key.
    pub key: Key,
    /// The canonical stored value.
    pub value: Value,
    /// Revision at which this value was written.
    pub revision: Revision,
}

/// Successful `set` outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct SetResult {
    /// The written entry, including its new revision.
    pub entry: Entry,
    /// `true` when the key did not previously exist.
    pub created: bool,
}

/// Successful `delete` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteResult {
    /// Revision the deleted entry held before removal.
    pub deleted_revision: Revision,
    /// Global revision assigned to the delete itself.
    pub revision: Revision,
}

/// Successful `list` outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct ListResult {
    /// Live entries in canonical (binary key) order.
    pub entries: Vec<Entry>,
    /// Current global revision counter.
    pub global_revision: u64,
}

/// Storage-independent command accepted by [`crate::KvApplication`].
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Write `value` at `key` subject to `expectation`.
    Set {
        key: Key,
        value: Value,
        expectation: SetExpectation,
    },
    Get {
        key: Key,
    },
    /// Remove `key` subject to `expectation`.
    Delete {
        key: Key,
        expectation: DeleteExpectation,
    },
    /// List every live entry in canonical order.
    List,
}

/// Storage-independent successful command result.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    /// Outcome of a `set`.
    Set(Box<SetResult>),
    /// Entry returned by a `get`.
    Get(Box<Entry>),
    /// Outcome of a `delete`.
    Delete(DeleteResult),
    /// Snapshot returned by a `list`.
    List(ListResult),
}

/// Milestone 1 TODO: parse and normalize the restricted JSON value model.
pub fn parse_json_value(_input: &str) -> Result<Value, KvError> {
    Err(KvError::incomplete("restricted JSON parsing"))
}
