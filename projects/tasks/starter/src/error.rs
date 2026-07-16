use std::error::Error as StdError;

use thiserror::Error;

pub type TaskResult<T> = Result<T, TaskError>;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("incomplete project capability: {capability}")]
    Incomplete { capability: &'static str },
    #[error("{message}")]
    Validation { field: String, message: String },
    #[error("task {id} was not found")]
    NotFound { id: i64 },
    #[error("task storage {operation} failed: {source}")]
    Storage {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
}

impl TaskError {
    #[must_use]
    pub const fn incomplete(capability: &'static str) -> Self {
        Self::Incomplete { capability }
    }

    #[must_use]
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn not_found(id: i64) -> Self {
        Self::NotFound { id }
    }

    pub fn storage(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Storage {
            operation: operation.into(),
            source: Box::new(source),
        }
    }

    #[must_use]
    pub const fn incomplete_capability(&self) -> Option<&'static str> {
        match self {
            Self::Incomplete { capability } => Some(capability),
            _ => None,
        }
    }

    #[must_use]
    pub fn validation_details(&self) -> Option<(&str, &str)> {
        match self {
            Self::Validation { field, message } => Some((field.as_str(), message.as_str())),
            _ => None,
        }
    }

    #[must_use]
    pub const fn not_found_id(&self) -> Option<i64> {
        match self {
            Self::NotFound { id } => Some(*id),
            _ => None,
        }
    }

    #[must_use]
    pub fn storage_operation(&self) -> Option<&str> {
        match self {
            Self::Storage { operation, .. } => Some(operation),
            _ => None,
        }
    }
}
