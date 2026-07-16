//! Injectable file-tree traversal and reading capability.

use crate::domain::portable_relative_path;
use crate::{ErrorCode, IndexError, IssueCode, RootSpec};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
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
        root: &'a RootSpec,
    ) -> Result<Box<dyn Iterator<Item = Result<TreeEntry, FileIssue>> + 'a>, IndexError> {
        Ok(Box::new(TreeWalker::new(root)?))
    }

    fn read(&self, entry: &TreeEntry, max_bytes: u64) -> Result<Vec<u8>, FileIssue> {
        let path = entry.relative_path.clone();
        let metadata = fs::symlink_metadata(&entry.host_path)
            .map_err(|source| file_io_issue(path.clone(), source))?;
        if metadata.file_type().is_symlink() {
            return Err(FileIssue::message(
                IssueCode::SymlinkSkipped,
                path,
                IssueCode::SymlinkSkipped.message(),
            ));
        }
        if metadata.len() > max_bytes {
            return Err(FileIssue::message(
                IssueCode::FileTooLarge,
                path,
                IssueCode::FileTooLarge.message(),
            ));
        }

        let file =
            File::open(&entry.host_path).map_err(|source| file_io_issue(path.clone(), source))?;
        let mut bytes = Vec::new();
        file.take(max_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|source| file_io_issue(path.clone(), source))?;
        if bytes.len() as u64 > max_bytes {
            return Err(FileIssue::message(
                IssueCode::FileTooLarge,
                path,
                IssueCode::FileTooLarge.message(),
            ));
        }
        Ok(bytes)
    }
}

struct DirectoryFrame {
    relative_path: Option<String>,
    entries: std::vec::IntoIter<Result<fs::DirEntry, io::Error>>,
}

struct TreeWalker {
    root_name: String,
    root_path: PathBuf,
    stack: Vec<DirectoryFrame>,
}

impl TreeWalker {
    fn new(root: &RootSpec) -> Result<Self, IndexError> {
        let metadata = fs::symlink_metadata(root.path())
            .map_err(|source| IndexError::io(ErrorCode::InvalidRoot, root.path(), source))?;
        if metadata.file_type().is_symlink() {
            return Err(IndexError::io(
                ErrorCode::InvalidRoot,
                root.path(),
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "root changed into a symbolic link",
                ),
            ));
        }
        let entries = sorted_directory(root.path())
            .map_err(|source| IndexError::io(ErrorCode::InvalidRoot, root.path(), source))?;
        Ok(Self {
            root_name: root.name().to_owned(),
            root_path: root.path().to_owned(),
            stack: vec![DirectoryFrame {
                relative_path: None,
                entries,
            }],
        })
    }
}

impl Iterator for TreeWalker {
    type Item = Result<TreeEntry, FileIssue>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (entry, parent_path) = {
                let frame = self.stack.last_mut()?;
                (frame.entries.next(), frame.relative_path.clone())
            };
            let Some(entry) = entry else {
                self.stack.pop();
                continue;
            };
            let entry = match entry {
                Ok(entry) => entry,
                Err(source) => {
                    return Some(Err(FileIssue::Io {
                        code: IssueCode::EntryUnreadable,
                        path: parent_path.or_else(|| Some("<entry>".to_owned())),
                        source,
                    }));
                }
            };

            let host_path = entry.path();
            let relative_path = portable_relative_path(&self.root_path, &host_path);
            if relative_path.is_none() {
                return Some(Ok(TreeEntry {
                    root: self.root_name.clone(),
                    host_path,
                    relative_path: None,
                    kind: TreeEntryKind::Other,
                }));
            }
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(source) => {
                    return Some(Err(FileIssue::Io {
                        code: IssueCode::EntryUnreadable,
                        path: relative_path,
                        source,
                    }));
                }
            };
            let kind = if file_type.is_symlink() {
                TreeEntryKind::Symlink
            } else if file_type.is_dir() {
                TreeEntryKind::Directory
            } else if file_type.is_file() {
                TreeEntryKind::RegularFile
            } else {
                TreeEntryKind::Other
            };
            let mut tree_entry = TreeEntry {
                root: self.root_name.clone(),
                host_path: host_path.clone(),
                relative_path: relative_path.clone(),
                kind,
            };
            if kind != TreeEntryKind::Directory {
                return Some(Ok(tree_entry));
            }

            match fs::symlink_metadata(&host_path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    tree_entry.kind = TreeEntryKind::Symlink;
                    return Some(Ok(tree_entry));
                }
                Err(source) => {
                    return Some(Err(FileIssue::Io {
                        code: IssueCode::EntryUnreadable,
                        path: relative_path,
                        source,
                    }));
                }
                Ok(_) => {}
            }
            match sorted_directory(&host_path) {
                Ok(entries) => self.stack.push(DirectoryFrame {
                    relative_path,
                    entries,
                }),
                Err(source) => {
                    return Some(Err(FileIssue::Io {
                        code: IssueCode::EntryUnreadable,
                        path: relative_path,
                        source,
                    }));
                }
            }
        }
    }
}

fn sorted_directory(
    directory: &Path,
) -> io::Result<std::vec::IntoIter<Result<fs::DirEntry, io::Error>>> {
    let mut entries = fs::read_dir(directory)?.collect::<Vec<_>>();
    entries.sort_by(|left, right| match (left, right) {
        (Ok(left), Ok(right)) => left.file_name().cmp(&right.file_name()),
        (Err(_), Ok(_)) => std::cmp::Ordering::Less,
        (Ok(_), Err(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => std::cmp::Ordering::Equal,
    });
    Ok(entries.into_iter())
}

fn file_io_issue(path: Option<String>, source: io::Error) -> FileIssue {
    let code = if source.kind() == io::ErrorKind::NotFound {
        IssueCode::FileDisappeared
    } else {
        IssueCode::FileUnreadable
    };
    FileIssue::Io { code, path, source }
}
