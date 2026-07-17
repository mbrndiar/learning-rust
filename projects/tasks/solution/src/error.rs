use std::error::Error as StdError;

use serde_json::Value;
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
    #[error("internal operation {operation} failed: {source}")]
    Internal {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    #[error("{message}")]
    ClientConfiguration { field: String, message: String },
    #[error("API returned {status} {code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
        details: Option<Value>,
    },
    #[error("unexpected server response: {message}")]
    UnexpectedResponse { message: String },
    #[error("request failed")]
    Connection {
        timeout: bool,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    #[error("server lifecycle {operation} failed: {source}")]
    Lifecycle {
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

    pub fn internal(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            operation: operation.into(),
            source: Box::new(source),
        }
    }

    #[must_use]
    pub fn client_configuration(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ClientConfiguration {
            field: field.into(),
            message: message.into(),
        }
    }

    #[must_use]
    pub fn api(
        status: u16,
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<Value>,
    ) -> Self {
        Self::Api {
            status,
            code: code.into(),
            message: message.into(),
            details,
        }
    }

    #[must_use]
    pub fn unexpected_response(message: impl Into<String>) -> Self {
        Self::UnexpectedResponse {
            message: message.into(),
        }
    }

    pub fn connection(source: impl StdError + Send + Sync + 'static, timeout: bool) -> Self {
        Self::Connection {
            timeout,
            source: Box::new(source),
        }
    }

    pub fn lifecycle(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Lifecycle {
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

    #[must_use]
    pub fn api_details(&self) -> Option<(u16, &str, &str, Option<&Value>)> {
        match self {
            Self::Api {
                status,
                code,
                message,
                details,
            } => Some((*status, code.as_str(), message.as_str(), details.as_ref())),
            _ => None,
        }
    }

    #[must_use]
    pub fn unexpected_response_message(&self) -> Option<&str> {
        match self {
            Self::UnexpectedResponse { message } => Some(message),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_connection(&self) -> bool {
        matches!(self, Self::Connection { .. })
    }

    #[must_use]
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::Connection { timeout: true, .. })
    }

    #[must_use]
    pub fn client_configuration_details(&self) -> Option<(&str, &str)> {
        match self {
            Self::ClientConfiguration { field, message } => {
                Some((field.as_str(), message.as_str()))
            }
            _ => None,
        }
    }
}
