//! Validated domain values and deterministic index result shapes.

use crate::IndexError;
use std::path::{Path, PathBuf};

/// A named directory root supplied as `NAME=PATH`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSpec {
    name: String,
    path: PathBuf,
}

impl RootSpec {
    /// Parses and validates one root specification.
    pub fn parse(_value: &str) -> Result<Self, IndexError> {
        Err(IndexError::incomplete("root validation"))
    }

    /// Returns the portable root name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the host path used for traversal.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// One normalized exact-match search term.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SearchTerm(String);

impl SearchTerm {
    /// Parses and validates a search term with the index tokenizer.
    pub fn parse(_value: &str) -> Result<Self, IndexError> {
        Err(IndexError::incomplete("search term validation"))
    }

    /// Borrows the normalized term.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A positive contiguous document identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(u64);

impl DocumentId {
    /// Validates a positive document identifier.
    pub fn new(_value: u64) -> Result<Self, IndexError> {
        Err(IndexError::incomplete("document id validation"))
    }

    /// Returns the identifier value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Settings recorded in a version-1 index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexSettings {
    pub extensions: Vec<String>,
    pub max_bytes: u64,
}

/// One normalized term count within a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermCount {
    pub term: String,
    pub count: u64,
}

/// One fully indexed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedDocument {
    pub id: DocumentId,
    pub root: String,
    pub path: String,
    pub bytes: u64,
    pub terms: Vec<TermCount>,
}

/// Stable recoverable issue codes recorded in an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueCode {
    EntryUnreadable,
    FileUnreadable,
    FileDisappeared,
    FileTooLarge,
    NonUtf8Content,
    NonUtf8Path,
    SymlinkSkipped,
    TokenTooLong,
}

impl IssueCode {
    /// Returns the stable snake-case code stored in the index.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EntryUnreadable => "entry_unreadable",
            Self::FileUnreadable => "file_unreadable",
            Self::FileDisappeared => "file_disappeared",
            Self::FileTooLarge => "file_too_large",
            Self::NonUtf8Content => "non_utf8_content",
            Self::NonUtf8Path => "non_utf8_path",
            Self::SymlinkSkipped => "symlink_skipped",
            Self::TokenTooLong => "token_too_long",
        }
    }
}

/// One recoverable path issue emitted by indexing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexIssue {
    pub root: String,
    pub path: Option<String>,
    pub code: IssueCode,
    pub message: String,
}

/// Complete versioned index data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexData {
    pub schema_version: u64,
    pub settings: IndexSettings,
    pub roots: Vec<String>,
    pub documents: Vec<IndexedDocument>,
    pub issues: Vec<IndexIssue>,
}

/// Validated search input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub terms: Vec<SearchTerm>,
    pub path_prefix: Option<String>,
    pub limit: usize,
}

/// Document fields exposed by a search result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSummary {
    pub id: DocumentId,
    pub root: String,
    pub path: String,
    pub bytes: u64,
}

/// One matching document and its requested term counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub document: DocumentSummary,
    pub term_counts: Vec<TermCount>,
}

/// Complete deterministic search response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub query: SearchQuery,
    pub matches: Vec<SearchMatch>,
}
