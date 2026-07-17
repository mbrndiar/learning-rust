//! Typed errors for the comparative capstone boundary.
//!
//! The finished contract requires a rich, normative failure taxonomy (category,
//! details object, and process exit code, per `spec/SPEC.md` §6). This scaffold ships
//! only the variants needed to compile — a milestone marker plus generic I/O and JSON
//! wrappers — and stub category/details/exit-code methods you extend as milestones
//! land.

use serde_json::{Value, json};
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Error returned by scaffold and future key/value operations.
#[derive(Debug, Error)]
pub enum KvError {
    /// The named capability belongs to a later implementation milestone.
    #[error("{capability} is not implemented yet")]
    Incomplete { capability: &'static str },
    /// A filesystem operation failed while accessing a path.
    #[error("cannot {operation} {}: {source}", path.display())]
    Io {
        /// Short label for the attempted operation (e.g. "open", "read").
        operation: &'static str,
        /// Path the operation targeted.
        path: PathBuf,
        /// Underlying OS error.
        #[source]
        source: io::Error,
    },
    /// JSON decoding or encoding failed.
    #[error("cannot process JSON for {context}: {source}")]
    Json {
        /// Where the JSON failure occurred.
        context: &'static str,
        /// Underlying serde_json error.
        #[source]
        source: serde_json::Error,
    },
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
            Self::Io { .. } | Self::Json { .. } => None,
        }
    }

    /// Milestone TODO: return the frozen contract's process exit code.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Incomplete { .. } | Self::Io { .. } | Self::Json { .. } => 5,
        }
    }

    /// Milestone TODO: return the frozen contract's error category.
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Self::Incomplete { .. } => "incomplete",
            Self::Io { .. } => "storage_error",
            Self::Json { .. } => "invalid_json",
        }
    }

    /// Milestone TODO: return the frozen contract's exact details object.
    #[must_use]
    pub fn details(&self) -> Value {
        match self {
            Self::Incomplete { capability } => json!({"capability": capability}),
            Self::Io { operation, .. } => {
                json!({"operation": operation, "reason": "storage_failure"})
            }
            Self::Json { .. } => json!({"reason": "syntax"}),
        }
    }
}
