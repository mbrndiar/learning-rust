//! Milestone 2: injectable traversal and reading.

use crate::{IndexError, IssueCode, RootSpec};
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Kind of entry observed without following symlinks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeEntryKind {
    Directory,
    RegularFile,
    Symlink,
    Other,
}

/// One host entry plus its portable root-relative path when representable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    pub root: String,
    pub host_path: PathBuf,
    pub relative_path: Option<String>,
    pub kind: TreeEntryKind,
}

/// Recoverable failure while inspecting or reading one entry.
#[derive(Debug, Error)]
pub enum FileIssue {
    #[error("{capability} is not implemented yet")]
    Incomplete { capability: &'static str },
    #[error("{} at {path:?}: {source}", code.as_str())]
    Io {
        code: IssueCode,
        path: Option<String>,
        #[source]
        source: io::Error,
    },
    #[error("{} at {path:?}: {message}", code.as_str())]
    Message {
        code: IssueCode,
        path: Option<String>,
        message: String,
    },
    #[error("fatal worker failure: {message}")]
    Fatal { message: String },
}

impl FileIssue {
    /// Constructs a typed scaffold failure for an unfinished file capability.
    #[must_use]
    pub const fn incomplete(capability: &'static str) -> Self {
        Self::Incomplete { capability }
    }

    /// Constructs a deterministic recoverable issue.
    #[must_use]
    pub fn message(code: IssueCode, path: Option<String>, message: impl Into<String>) -> Self {
        Self::Message {
            code,
            path,
            message: message.into(),
        }
    }

    /// Constructs a fatal fake/provider failure used by the worker protocol.
    #[must_use]
    pub fn fatal(message: impl Into<String>) -> Self {
        Self::Fatal {
            message: message.into(),
        }
    }

    /// Returns the recoverable issue code, if this is not fatal.
    #[must_use]
    pub const fn code(&self) -> Option<IssueCode> {
        match self {
            Self::Io { code, .. } | Self::Message { code, .. } => Some(*code),
            Self::Incomplete { .. } | Self::Fatal { .. } => None,
        }
    }

    /// Returns the portable path carried by a recoverable issue.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        match self {
            Self::Io { path, .. } | Self::Message { path, .. } => path.as_deref(),
            Self::Incomplete { .. } | Self::Fatal { .. } => None,
        }
    }
}

/// Traversal and read seam used by real and deterministic fake trees.
pub trait FileTree: Send + Sync {
    fn entries<'a>(
        &'a self,
        root: &'a RootSpec,
    ) -> Result<Box<dyn Iterator<Item = Result<TreeEntry, FileIssue>> + 'a>, IndexError>;
    fn read(&self, entry: &TreeEntry, max_bytes: u64) -> Result<Vec<u8>, FileIssue>;
}

/// Standard-library filesystem implementation that never follows symlinks.
#[derive(Debug, Clone, Copy, Default)]
pub struct StdFileTree;

impl FileTree for StdFileTree {
    fn entries<'a>(
        &'a self,
        _root: &'a RootSpec,
    ) -> Result<Box<dyn Iterator<Item = Result<TreeEntry, FileIssue>> + 'a>, IndexError> {
        todo!("milestone 2: walk without following symlinks")
    }

    fn read(&self, _entry: &TreeEntry, _max_bytes: u64) -> Result<Vec<u8>, FileIssue> {
        todo!("milestone 2: bound reads and classify recoverable failures")
    }
}
