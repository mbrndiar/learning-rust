//! Source-preserving fatal errors for the file indexer.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Stable observable error codes from the idiomatic capstone specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidArgument,
    InvalidRoot,
    DuplicateRoot,
    InvalidExtension,
    InvalidSearchTerm,
    InvalidPathPrefix,
    IndexNotFound,
    IndexCorrupt,
    UnsupportedIndexVersion,
    IndexReadFailed,
    IndexWriteFailed,
    WorkerFailed,
    Cancelled,
}

impl ErrorCode {
    /// Returns the stable snake-case code used by JSON diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArgument => "invalid_argument",
            Self::InvalidRoot => "invalid_root",
            Self::DuplicateRoot => "duplicate_root",
            Self::InvalidExtension => "invalid_extension",
            Self::InvalidSearchTerm => "invalid_search_term",
            Self::InvalidPathPrefix => "invalid_path_prefix",
            Self::IndexNotFound => "index_not_found",
            Self::IndexCorrupt => "index_corrupt",
            Self::UnsupportedIndexVersion => "unsupported_index_version",
            Self::IndexReadFailed => "index_read_failed",
            Self::IndexWriteFailed => "index_write_failed",
            Self::WorkerFailed => "worker_failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Fatal failure returned across the public indexer boundary.
#[derive(Debug, Error)]
pub enum IndexError {
    /// The named capability is intentionally unfinished in the guided starter.
    #[error("{capability} is not implemented yet")]
    Incomplete { capability: &'static str },
    /// A validated operation failed without an underlying provider error.
    #[error("{}: {message}", code.as_str())]
    Contract { code: ErrorCode, message: String },
    /// A filesystem operation failed and preserves its original source.
    #[error("{} while accessing {}: {source}", code.as_str(), path.display())]
    Io {
        code: ErrorCode,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// JSON processing failed and preserves the Serde source.
    #[error("{} while processing JSON: {source}", code.as_str())]
    Json {
        code: ErrorCode,
        #[source]
        source: serde_json::Error,
    },
}

impl IndexError {
    /// Constructs a typed scaffold failure for an unfinished capability.
    #[must_use]
    pub const fn incomplete(capability: &'static str) -> Self {
        Self::Incomplete { capability }
    }

    /// Constructs a stable contract failure.
    #[must_use]
    pub fn contract(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Contract {
            code,
            message: message.into(),
        }
    }

    /// Constructs a source-preserving filesystem failure.
    #[must_use]
    pub fn io(code: ErrorCode, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            code,
            path: path.into(),
            source,
        }
    }

    /// Constructs a source-preserving JSON failure.
    #[must_use]
    pub const fn json(code: ErrorCode, source: serde_json::Error) -> Self {
        Self::Json { code, source }
    }

    /// Returns the unfinished capability when this is a scaffold failure.
    #[must_use]
    pub const fn incomplete_capability(&self) -> Option<&'static str> {
        match self {
            Self::Incomplete { capability } => Some(capability),
            Self::Contract { .. } | Self::Io { .. } | Self::Json { .. } => None,
        }
    }

    /// Returns the stable observable error code.
    #[must_use]
    pub const fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::Incomplete { .. } => None,
            Self::Contract { code, .. } | Self::Io { code, .. } | Self::Json { code, .. } => {
                Some(*code)
            }
        }
    }

    /// Returns the process exit required by the CLI contract.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Incomplete { .. } => 5,
            Self::Io {
                code: ErrorCode::InvalidRoot,
                ..
            } => 3,
            Self::Contract {
                code: ErrorCode::Cancelled,
                ..
            } => 130,
            Self::Contract {
                code: ErrorCode::WorkerFailed | ErrorCode::IndexWriteFailed,
                ..
            }
            | Self::Io {
                code: ErrorCode::WorkerFailed | ErrorCode::IndexWriteFailed,
                ..
            }
            | Self::Json {
                code: ErrorCode::WorkerFailed | ErrorCode::IndexWriteFailed,
                ..
            } => 5,
            Self::Contract {
                code:
                    ErrorCode::IndexNotFound
                    | ErrorCode::IndexCorrupt
                    | ErrorCode::UnsupportedIndexVersion
                    | ErrorCode::IndexReadFailed,
                ..
            }
            | Self::Io {
                code:
                    ErrorCode::IndexNotFound
                    | ErrorCode::IndexCorrupt
                    | ErrorCode::UnsupportedIndexVersion
                    | ErrorCode::IndexReadFailed,
                ..
            }
            | Self::Json {
                code:
                    ErrorCode::IndexNotFound
                    | ErrorCode::IndexCorrupt
                    | ErrorCode::UnsupportedIndexVersion
                    | ErrorCode::IndexReadFailed,
                ..
            } => 4,
            Self::Contract { .. } | Self::Io { .. } | Self::Json { .. } => 2,
        }
    }
}
