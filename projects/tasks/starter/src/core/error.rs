//! Errors produced by task rules, application services, and repositories.
//!
//! [`TaskError`] is deliberately limited to failures that cross the core
//! application boundary. Client transport/configuration failures and server
//! lifecycle failures have their own adapter-specific error types.

use std::error::Error as StdError;

use thiserror::Error;

/// Convenient alias for a core operation returning [`TaskError`].
pub type TaskResult<T> = Result<T, TaskError>;

/// A domain, application, or persistence failure.
#[derive(Debug, Error)]
pub enum TaskError {
    /// An intentionally unimplemented capability in the starter scaffold.
    #[error("incomplete project capability: {capability}")]
    Incomplete { capability: &'static str },
    /// A domain value that violates the rules; maps to HTTP `422`.
    #[error("{message}")]
    Validation { field: String, message: String },
    /// A task ID that does not exist; maps to HTTP `404`.
    #[error("task {id} was not found")]
    NotFound { id: i64 },
    /// A persistence failure; its source is logged but never exposed over HTTP.
    #[error("task storage {operation} failed: {source}")]
    Storage {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    /// An unexpected application failure such as a panicked blocking task.
    #[error("internal operation {operation} failed: {source}")]
    Internal {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
}

impl TaskError {
    /// Marks an unfinished capability; `capability` names the missing milestone.
    #[must_use]
    pub const fn incomplete(capability: &'static str) -> Self {
        Self::Incomplete { capability }
    }

    /// Builds a validation error tagged with the offending `field`.
    #[must_use]
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Builds a not-found error for `id`.
    #[must_use]
    pub const fn not_found(id: i64) -> Self {
        Self::NotFound { id }
    }

    /// Wraps a persistence failure, preserving `source` for diagnostics.
    pub fn storage(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Storage {
            operation: operation.into(),
            source: Box::new(source),
        }
    }

    /// Wraps an unexpected application failure, preserving `source`.
    pub fn internal(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            operation: operation.into(),
            source: Box::new(source),
        }
    }

    /// Returns the missing capability if this is [`TaskError::Incomplete`].
    #[must_use]
    pub const fn incomplete_capability(&self) -> Option<&'static str> {
        match self {
            Self::Incomplete { capability } => Some(capability),
            _ => None,
        }
    }

    /// Returns the `(field, message)` pair if this is a validation error.
    #[must_use]
    pub fn validation_details(&self) -> Option<(&str, &str)> {
        match self {
            Self::Validation { field, message } => Some((field.as_str(), message.as_str())),
            _ => None,
        }
    }

    /// Returns the missing ID if this is a not-found error.
    #[must_use]
    pub const fn not_found_id(&self) -> Option<i64> {
        match self {
            Self::NotFound { id } => Some(*id),
            _ => None,
        }
    }

    /// Returns the operation name if this is a storage error.
    #[must_use]
    pub fn storage_operation(&self) -> Option<&str> {
        match self {
            Self::Storage { operation, .. } => Some(operation),
            _ => None,
        }
    }
}
