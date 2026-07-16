//! Typed errors and normative response metadata for the comparative capstone.

use serde_json::{Value, json};
use thiserror::Error;

/// Error returned by key/value operations.
#[derive(Debug, Error)]
pub enum KvError {
    /// The named capability belongs to a later implementation milestone.
    #[error("{capability} is not implemented yet")]
    Incomplete { capability: &'static str },
    #[error("invalid command line")]
    Usage,
    #[error("invalid {field}: {reason}")]
    InvalidArgument {
        field: &'static str,
        reason: &'static str,
    },
    #[error("invalid JSON syntax")]
    InvalidJson,
    #[error("invalid JSON value: {reason}")]
    InvalidValue { reason: &'static str },
    #[error("expectation conflict for {key}")]
    ConflictAbsent { key: String, actual: u64 },
    #[error("expectation conflict for {key}")]
    ConflictExact {
        key: String,
        expected: u64,
        actual: Option<u64>,
    },
    #[error("key not found: {key}")]
    NotFound { key: String },
    #[error("SQLite remained busy")]
    Busy,
    #[error("unsupported schema version {found}")]
    UnsupportedSchema { found: i64 },
    #[error("invalid storage: {reason}")]
    InvalidStorage {
        reason: &'static str,
        key: Option<String>,
    },
    #[error("global revision is exhausted")]
    RevisionExhausted,
    #[error("storage operation failed: {operation}")]
    Storage { operation: &'static str },
}

impl KvError {
    /// Constructs a typed scaffold failure for an unfinished capability.
    #[must_use]
    pub const fn incomplete(capability: &'static str) -> Self {
        Self::Incomplete { capability }
    }

    /// Returns the unfinished capability when this is a scaffold failure.
    #[must_use]
    pub const fn incomplete_capability(&self) -> Option<&'static str> {
        match self {
            Self::Incomplete { capability } => Some(capability),
            _ => None,
        }
    }

    /// Returns the normative process exit code.
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
            | Self::Storage { .. }
            | Self::Incomplete { .. } => 5,
        }
    }

    /// Returns the normative error category.
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Self::Incomplete { .. } => "storage_error",
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
            Self::Incomplete { .. } => {
                json!({"operation": "write", "reason": "storage_failure"})
            }
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
