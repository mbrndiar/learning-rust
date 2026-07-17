//! Errors produced by client configuration, transport, and response decoding.

use std::error::Error as StdError;

use serde_json::Value;
use thiserror::Error;

use crate::TaskError;

/// Convenient alias for a client operation returning [`ClientError`].
pub type ClientResult<T> = Result<T, ClientError>;

/// A failure at the outbound HTTP or CLI boundary.
#[derive(Debug, Error)]
pub enum ClientError {
    /// A shared task validation or starter-scaffold failure.
    #[error(transparent)]
    Task(#[from] TaskError),
    /// Invalid local client configuration such as a base URL or timeout.
    #[error("{message}")]
    Configuration { field: String, message: String },
    /// A documented API error decoded from a server response.
    #[error("API returned {status} {code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
        details: Option<Value>,
    },
    /// A server response the client could not trust.
    #[error("unexpected server response: {message}")]
    UnexpectedResponse { message: String },
    /// A transport failure; `timeout` distinguishes deadline expiry.
    #[error("request failed")]
    Connection {
        timeout: bool,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    /// An unexpected client-side serialization or rendering failure.
    #[error("internal operation {operation} failed: {source}")]
    Internal {
        operation: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
}

impl ClientError {
    /// Builds a configuration error tagged with the offending `field`.
    #[must_use]
    pub fn configuration(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Configuration {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Builds a decoded API error.
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

    /// Builds an error for a response that violated the wire contract.
    #[must_use]
    pub fn unexpected_response(message: impl Into<String>) -> Self {
        Self::UnexpectedResponse {
            message: message.into(),
        }
    }

    /// Wraps a transport failure.
    pub fn connection(source: impl StdError + Send + Sync + 'static, timeout: bool) -> Self {
        Self::Connection {
            timeout,
            source: Box::new(source),
        }
    }

    /// Wraps an unexpected client-side failure.
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

    /// Returns decoded API details, if present.
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

    /// Returns why a response was untrustworthy.
    #[must_use]
    pub fn unexpected_response_message(&self) -> Option<&str> {
        match self {
            Self::UnexpectedResponse { message } => Some(message),
            _ => None,
        }
    }

    /// Whether this is a transport failure.
    #[must_use]
    pub const fn is_connection(&self) -> bool {
        matches!(self, Self::Connection { .. })
    }

    /// Whether this is a transport timeout.
    #[must_use]
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::Connection { timeout: true, .. })
    }

    /// Returns invalid configuration details, if present.
    #[must_use]
    pub fn configuration_details(&self) -> Option<(&str, &str)> {
        match self {
            Self::Configuration { field, message } => Some((field.as_str(), message.as_str())),
            _ => None,
        }
    }

    /// Returns shared validation details, if this wraps a validation error.
    #[must_use]
    pub fn validation_details(&self) -> Option<(&str, &str)> {
        self.task_error().and_then(TaskError::validation_details)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;
    use std::io;

    use super::*;

    #[test]
    fn categories_preserve_details_and_sources() {
        let api = ClientError::api(
            404,
            "not_found",
            "missing",
            Some(serde_json::json!({"id": 7})),
        );
        let (status, code, message, details) = api.api_details().expect("API details");
        assert_eq!((status, code, message), (404, "not_found", "missing"));
        assert_eq!(details.expect("API detail")["id"], 7);

        let malformed = ClientError::unexpected_response("wrong shape");
        assert_eq!(malformed.unexpected_response_message(), Some("wrong shape"));

        let connection = ClientError::connection(io::Error::other("offline"), true);
        assert!(connection.is_connection());
        assert!(connection.is_timeout());
        assert_eq!(
            connection.source().expect("connection source").to_string(),
            "offline"
        );

        let internal = ClientError::internal("encode", io::Error::other("invalid value"));
        assert_eq!(
            internal.source().expect("internal source").to_string(),
            "invalid value"
        );
    }
}
