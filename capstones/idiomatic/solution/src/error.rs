//! Source-preserving fatal errors for the file indexer.
//!
//! [`IndexError`] is the single fatal type crossing the public boundary. Its
//! variants separate *how* a failure happened — a validated contract violation, a
//! filesystem error, or a JSON error — while [`ErrorCode`] carries the *stable
//! observable* classification from the specification. Keeping the code orthogonal
//! to the variant lets the same code (for example `index_write_failed`) arise from
//! either an I/O or a JSON source while preserving that source for diagnostics.
//! [`IndexError::exit_code`] then collapses codes into the process exits the CLI
//! contract promises.

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

    /// Returns the stable observable error code.
    #[must_use]
    pub const fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::Contract { code, .. } | Self::Io { code, .. } | Self::Json { code, .. } => {
                Some(*code)
            }
        }
    }

    /// Returns the process exit required by the CLI contract.
    ///
    /// The mapping groups codes by phase rather than by variant: a preflight root
    /// failure is `3`, an index read/corruption/version failure is `4`, a write or
    /// worker failure is `5`, cancellation is `130`, and any remaining validated
    /// argument error is `2`.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            // Only a root/traversal preflight failure is exit 3.
            Self::Io {
                code: ErrorCode::InvalidRoot,
                ..
            } => 3,
            // A cancelled build uses the conventional 128 + SIGINT exit.
            Self::Contract {
                code: ErrorCode::Cancelled,
                ..
            } => 130,
            // Write/worker protocol failures (from any source) are exit 5.
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
            // Read-side index failures (missing/corrupt/unsupported/read) are exit 4.
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
            // Every remaining validated argument error is a Clap-style usage exit.
            Self::Contract { .. } | Self::Io { .. } | Self::Json { .. } => 2,
        }
    }
}
