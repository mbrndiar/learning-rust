//! `TaskError`: the one typed error that crosses every boundary in the project.
//!
//! A single enum classifies every failure so that adapters can translate it
//! without parsing English messages. The HTTP boundary maps variants to status
//! codes and the CLI maps them to exit codes, while the source-preserving
//! variants ([`TaskError::Storage`], [`TaskError::Internal`],
//! [`TaskError::Connection`], [`TaskError::Lifecycle`]) keep the underlying
//! cause inside the process for diagnostics without leaking it to clients.

use std::error::Error as StdError;

use serde_json::Value;
use thiserror::Error;

/// Convenient alias for a result whose error is [`TaskError`].
pub type TaskResult<T> = Result<T, TaskError>;

/// Every failure the project can produce, grouped by how callers react to it.
#[derive(Debug, Error)]
pub enum TaskError {
    /// An intentionally unimplemented capability in the starter scaffold.
    #[error("incomplete project capability: {capability}")]
    Incomplete { capability: &'static str },
    /// A domain or request value that violates the rules; maps to HTTP `422`.
    #[error("{message}")]
    Validation { field: String, message: String },
    /// A task ID that does not exist; maps to HTTP `404`.
    #[error("task {id} was not found")]
    NotFound { id: i64 },
    /// A persistence failure; the `source` is kept for logs but sanitized to a
    /// `500` before reaching a client.
    #[error("task storage {operation} failed: {source}")]
    Storage {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    /// An unexpected internal failure such as a panicked blocking task; maps to
    /// `500`.
    #[error("internal operation {operation} failed: {source}")]
    Internal {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    /// Invalid client configuration (base URL or timeout); surfaces as a usage
    /// error in the CLI.
    #[error("{message}")]
    ClientConfiguration { field: String, message: String },
    /// A documented API error decoded from a server response by the client.
    #[error("API returned {status} {code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
        details: Option<Value>,
    },
    /// A server response the client could not trust (wrong status, type, or
    /// shape).
    #[error("unexpected server response: {message}")]
    UnexpectedResponse { message: String },
    /// A transport failure; `timeout` distinguishes a deadline from other
    /// connection errors.
    #[error("request failed")]
    Connection {
        timeout: bool,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    /// A server bind/serve/shutdown failure in a composition root.
    #[error("server lifecycle {operation} failed: {source}")]
    Lifecycle {
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

    /// Wraps an unexpected internal failure, preserving `source`.
    pub fn internal(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            operation: operation.into(),
            source: Box::new(source),
        }
    }

    /// Builds a client-configuration error tagged with the offending `field`.
    #[must_use]
    pub fn client_configuration(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ClientConfiguration {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Builds a decoded API error carrying the HTTP status, code, and optional
    /// details.
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

    /// Builds an unexpected-response error describing why a response was
    /// untrustworthy.
    #[must_use]
    pub fn unexpected_response(message: impl Into<String>) -> Self {
        Self::UnexpectedResponse {
            message: message.into(),
        }
    }

    /// Wraps a transport failure; set `timeout` when the request hit its
    /// deadline.
    pub fn connection(source: impl StdError + Send + Sync + 'static, timeout: bool) -> Self {
        Self::Connection {
            timeout,
            source: Box::new(source),
        }
    }

    /// Wraps a server lifecycle failure, preserving `source`.
    pub fn lifecycle(
        operation: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Lifecycle {
            operation: operation.into(),
            source: Box::new(source),
        }
    }

    /// Returns the missing capability name if this is an [`TaskError::Incomplete`].
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

    /// Returns the decoded `(status, code, message, details)` if this is an API
    /// error.
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

    /// Returns the description if this is an unexpected-response error.
    #[must_use]
    pub fn unexpected_response_message(&self) -> Option<&str> {
        match self {
            Self::UnexpectedResponse { message } => Some(message),
            _ => None,
        }
    }

    /// Whether this is a transport (connection) error.
    #[must_use]
    pub const fn is_connection(&self) -> bool {
        matches!(self, Self::Connection { .. })
    }

    /// Whether this is a transport error caused by a timeout.
    #[must_use]
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::Connection { timeout: true, .. })
    }

    /// Returns the `(field, message)` pair if this is a client-configuration
    /// error.
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
