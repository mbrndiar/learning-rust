//! Milestone 2: injectable traversal and reading.
//!
//! The [`FileTree`] trait is the seam that decouples indexing from the real
//! filesystem: production uses [`StdFileTree`], while tests inject deterministic
//! fakes to exercise error and cancellation paths. Two guarantees shape the
//! contract: symlinks are observed but never followed (so traversal cannot leave a
//! root or loop), and directory entries are yielded in sorted `file_name` order so
//! a walk is reproducible. Recoverable per-entry problems must surface as
//! [`FileIssue`] values rather than aborting the whole build.

use crate::{IndexError, IssueCode, RootSpec};
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Kind of entry observed without following symlinks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeEntryKind {
    /// A directory to descend into.
    Directory,
    /// A regular file eligible for reading.
    RegularFile,
    /// A symbolic link, recorded but never followed.
    Symlink,
    /// Any other node (device, socket, FIFO, non-UTF-8 path, ...).
    Other,
}

/// One host entry plus its portable root-relative path when representable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    /// Root name this entry was discovered under.
    pub root: String,
    /// Absolute host path used for reading and re-checking.
    pub host_path: PathBuf,
    /// Portable relative path, or `None` when it is not representable.
    pub relative_path: Option<String>,
    /// Observed entry kind (symlinks are not resolved).
    pub kind: TreeEntryKind,
}

/// Recoverable failure while inspecting or reading one entry.
#[derive(Debug, Error)]
pub enum FileIssue {
    /// An OS I/O error, classified into a stable [`IssueCode`].
    #[error("{} at {path:?}: {source}", code.as_str())]
    Io {
        code: IssueCode,
        path: Option<String>,
        #[source]
        source: io::Error,
    },
    /// A deterministic recoverable issue with a fixed message (e.g. symlink/too big).
    #[error("{} at {path:?}: {message}", code.as_str())]
    Message {
        code: IssueCode,
        path: Option<String>,
        message: String,
    },
    /// A non-recoverable provider failure that must abort the whole build.
    #[error("fatal worker failure: {message}")]
    Fatal { message: String },
}

impl FileIssue {
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
            Self::Fatal { .. } => None,
        }
    }

    /// Returns the portable path carried by a recoverable issue.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        match self {
            Self::Io { path, .. } | Self::Message { path, .. } => path.as_deref(),
            Self::Fatal { .. } => None,
        }
    }
}

/// Traversal and read seam used by real and deterministic fake trees.
///
/// Implementations must be `Send + Sync` so the builder can share one tree across
/// worker threads. `entries` streams a root lazily (a fatal setup error is
/// reported eagerly as [`IndexError`]); `read` returns the file bytes or a
/// recoverable [`FileIssue`].
pub trait FileTree: Send + Sync {
    /// Streams entries under `root` in deterministic order without following links.
    fn entries<'a>(
        &'a self,
        root: &'a RootSpec,
    ) -> Result<Box<dyn Iterator<Item = Result<TreeEntry, FileIssue>> + 'a>, IndexError>;
    /// Reads up to `max_bytes`, rejecting symlinks and over-large files as issues.
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
