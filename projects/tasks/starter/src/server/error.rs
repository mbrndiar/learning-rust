//! Errors produced while composing, binding, serving, or stopping a server.

use std::error::Error as StdError;

use thiserror::Error;

use crate::TaskError;

/// Convenient alias for a server operation returning [`ServerError`].
pub type ServerResult<T> = Result<T, ServerError>;

/// A failure at the inbound server lifecycle boundary.
#[derive(Debug, Error)]
pub enum ServerError {
    /// A core/application/storage failure encountered during composition.
    #[error(transparent)]
    Task(#[from] TaskError),
    /// A bind, serve, shutdown, or worker-thread lifecycle failure.
    #[error("server lifecycle {operation} failed: {source}")]
    Lifecycle {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    /// An unexpected server-composition failure.
    #[error("internal operation {operation} failed: {source}")]
    Internal {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
}

impl ServerError {
    /// Wraps a lifecycle failure, preserving its source.
    pub fn lifecycle(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Lifecycle {
            operation: operation.into(),
            source: Box::new(source),
        }
    }

    /// Wraps an unexpected server-composition failure.
    pub fn internal(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            operation: operation.into(),
            source: Box::new(source),
        }
    }

    /// Returns the wrapped core error, if present.
    #[must_use]
    pub const fn task_error(&self) -> Option<&TaskError> {
        match self {
            Self::Task(error) => Some(error),
            _ => None,
        }
    }
}
