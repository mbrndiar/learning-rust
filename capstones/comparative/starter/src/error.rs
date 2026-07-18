//! Typed errors and normative response metadata for the comparative capstone.
//!
//! The error taxonomy is complete scaffolding because it is part of the frozen
//! observable contract. Milestone bodies still fail visibly through a private
//! helper, but the public type stays identical to the finished solution.

use serde_json::{Value, json};
use thiserror::Error;

/// Error returned by key/value operations.
#[derive(Debug, Error)]
pub enum KvError {
    /// The command line did not match the exact grammar.
    #[error("invalid command line")]
    Usage,
    /// A named argument failed validation (e.g. key length, revision range).
    #[error("invalid {field}: {reason}")]
    InvalidArgument {
        field: &'static str,
        reason: &'static str,
    },
    /// The value argument was not syntactically valid JSON.
    #[error("invalid JSON syntax")]
    InvalidJson,
    /// The value parsed but violated the restricted value model.
    #[error("invalid JSON value: {reason}")]
    InvalidValue { reason: &'static str },
    /// `--expect absent` required no key, but one exists at revision `actual`.
    #[error("expectation conflict for {key}")]
    ConflictAbsent { key: String, actual: u64 },
    /// `--expect <revision>` required `expected`, but found `actual` (or absent).
    #[error("expectation conflict for {key}")]
    ConflictExact {
        key: String,
        expected: u64,
        actual: Option<u64>,
    },
    /// The key was absent for an operation that requires it.
    #[error("key not found: {key}")]
    NotFound { key: String },
    /// SQLite stayed busy past the configured timeout.
    #[error("SQLite remained busy")]
    Busy,
    /// The database used an unsupported schema version.
    #[error("unsupported schema version {found}")]
    UnsupportedSchema { found: i64 },
    /// Stored data violated an invariant the store must uphold.
    #[error("invalid storage: {reason}")]
    InvalidStorage {
        reason: &'static str,
        key: Option<String>,
    },
    /// The global revision counter reached the maximum safe integer.
    #[error("global revision is exhausted")]
    RevisionExhausted,
    /// A lower-level storage operation failed.
    #[error("storage operation failed: {operation}")]
    Storage { operation: &'static str },
}

impl KvError {
    /// Makes an unfinished starter operation fail through the real public
    /// storage-error category without adding a scaffold-only enum variant.
    pub(crate) const fn incomplete(capability: &'static str) -> Self {
        Self::Storage {
            operation: capability,
        }
    }

    /// Returns the normative process exit code.
    ///
    /// The spec fixes five buckets: `2` for any client-side usage/validation fault,
    /// `3` for an optimistic-concurrency conflict, `4` for a missing key, and `5`
    /// for every storage/internal failure.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Usage
            | Self::InvalidArgument { .. }
            | Self::InvalidJson
            | Self::InvalidValue { .. } => 2,
            Self::ConflictAbsent { .. } | Self::ConflictExact { .. } => 3,
            Self::NotFound { .. } => 4,
            Self::Busy
            | Self::UnsupportedSchema { .. }
            | Self::InvalidStorage { .. }
            | Self::RevisionExhausted
            | Self::Storage { .. } => 5,
        }
    }

    /// Returns the normative error category.
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Self::Usage => "usage",
            Self::InvalidArgument { .. } => "invalid_argument",
            Self::InvalidJson => "invalid_json",
            Self::InvalidValue { .. } => "invalid_value",
            Self::ConflictAbsent { .. } | Self::ConflictExact { .. } => "conflict",
            Self::NotFound { .. } => "not_found",
            Self::Busy => "busy",
            Self::UnsupportedSchema { .. } => "unsupported_schema",
            Self::InvalidStorage { .. } => "invalid_storage",
            Self::RevisionExhausted => "revision_exhausted",
            Self::Storage { .. } => "storage_error",
        }
    }

    /// Returns the exact normative error-details object.
    #[must_use]
    pub fn details(&self) -> Value {
        match self {
            Self::Usage => json!({"reason": "invalid_cli"}),
            Self::InvalidArgument { field, reason } => {
                json!({"field": field, "reason": reason})
            }
            Self::InvalidJson => json!({"reason": "syntax"}),
            Self::InvalidValue { reason } => json!({"reason": reason}),
            Self::ConflictAbsent { key, actual } => {
                json!({"key": key, "expected": "absent", "actual": actual})
            }
            Self::ConflictExact {
                key,
                expected,
                actual,
            } => json!({"key": key, "expected": expected, "actual": actual}),
            Self::NotFound { key } => json!({"key": key}),
            Self::Busy => json!({"timeout_ms": crate::store::BUSY_TIMEOUT_MS}),
            Self::UnsupportedSchema { found } => {
                json!({"found": found, "supported": 1})
            }
            Self::InvalidStorage { reason, key } => match key {
                Some(key) => json!({"reason": reason, "key": key}),
                None => json!({"reason": reason}),
            },
            Self::RevisionExhausted => json!({"maximum": crate::domain::MAX_SAFE_INTEGER}),
            Self::Storage { operation } => {
                json!({"operation": operation, "reason": "storage_failure"})
            }
        }
    }

    /// Converts this error to the exact failure envelope.
    #[must_use]
    pub fn envelope(&self) -> Value {
        json!({
            "ok": false,
            "error": {
                "category": self.category(),
                "details": self.details(),
            }
        })
    }
}
