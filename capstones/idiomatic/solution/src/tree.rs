//! Injectable file-tree traversal and reading capability.

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
    #[error("{code:?} at {path:?}: {source}")]
    Io {
        code: IssueCode,
        path: Option<PathBuf>,
        #[source]
        source: io::Error,
    },
}

impl FileIssue {
    /// Constructs a typed scaffold failure for an unfinished file capability.
    #[must_use]
    pub const fn incomplete(capability: &'static str) -> Self {
        Self::Incomplete { capability }
    }
}

/// Traversal and read seam used by real and deterministic fake trees.
pub trait FileTree: Send + Sync {
    fn entries<'a>(
        &'a self,
        root: &'a RootSpec,
    ) -> Result<Box<dyn Iterator<Item = TreeEntry> + 'a>, IndexError>;
    fn read(&self, entry: &TreeEntry, max_bytes: u64) -> Result<Vec<u8>, FileIssue>;
}
