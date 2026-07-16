//! Storage-independent values for the shared key/value contract.

use crate::KvError;
use serde_json::Value;

pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub const MAX_VALUE_BYTES: usize = 65_536;
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
    Any,
    Absent,
    Exact(Revision),
}

/// Conditional behavior supported by `delete`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteExpectation {
    Any,
    Exact(Revision),
}

/// One live key/value entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: Key,
    pub value: Value,
    pub revision: Revision,
}

/// Successful `set` outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct SetResult {
    pub entry: Entry,
    pub created: bool,
}

/// Successful `delete` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteResult {
    pub deleted_revision: Revision,
    pub revision: Revision,
}

/// Successful `list` outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct ListResult {
    pub entries: Vec<Entry>,
    pub global_revision: u64,
}

/// Storage-independent command accepted by [`crate::KvApplication`].
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Set {
        key: Key,
        value: Value,
        expectation: SetExpectation,
    },
    Get {
        key: Key,
    },
    Delete {
        key: Key,
        expectation: DeleteExpectation,
    },
    List,
}

/// Storage-independent successful command result.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    Set(Box<SetResult>),
    Get(Box<Entry>),
    Delete(DeleteResult),
    List(ListResult),
}

/// Milestone 1 TODO: parse and normalize the restricted JSON value model.
pub fn parse_json_value(_input: &str) -> Result<Value, KvError> {
    Err(KvError::incomplete("restricted JSON parsing"))
}
