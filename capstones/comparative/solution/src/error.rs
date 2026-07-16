//! Typed errors for the comparative capstone boundary.

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
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// JSON decoding or encoding failed.
    #[error("cannot process JSON for {context}: {source}")]
    Json {
        context: &'static str,
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
}
