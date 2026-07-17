//! Milestone 1: validated values and deterministic result shapes.
//!
//! This module is the rulebook you implement first. Private wrapper types must use
//! validating constructors. Aggregate index shapes expose fields for Serde, so
//! [`IndexData::validate`] must reject invalid combinations before they are trusted.
//! Two ideas run through the contracts below:
//!
//! * Portable vs. host paths. A `RootSpec` keeps a canonical host path for
//!   traversal/containment, while persisted documents store `/`-joined portable
//!   relative paths that must never escape their root.
//! * Determinism. Extensions, terms, documents, and issues must end up sorted and
//!   deduplicated so identical inputs serialize identically, and
//!   [`IndexData::validate`] must re-check those orderings after untrusted JSON.
//!
//! The field docs state the persisted contract; the `todo!()` bodies mark the work.

use crate::IndexError;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

/// A named, readable directory root supplied as `NAME=PATH`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSpec {
    name: String,
    path: PathBuf,
}

impl RootSpec {
    /// Parses, canonicalizes, and preflights one root specification.
    pub fn parse(_value: &str) -> Result<Self, IndexError> {
        todo!("milestone 1: validate and preflight NAME=PATH")
    }

    /// Returns the portable root name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the canonical host path used only for traversal and containment.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Parses roots and rejects duplicate names or canonical directories.
pub fn validate_roots(_values: &[String]) -> Result<Vec<RootSpec>, IndexError> {
    todo!("milestone 1: reject duplicate names and canonical paths")
}

/// One normalized exact-match search term.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SearchTerm(String);

impl SearchTerm {
    /// Parses and validates a search term with the index tokenizer.
    pub fn parse(_value: &str) -> Result<Self, IndexError> {
        todo!("milestone 1: normalize exactly one search token")
    }

    /// Borrows the normalized term.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A positive contiguous document identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct DocumentId(u64);

impl DocumentId {
    /// Validates a positive document identifier.
    pub fn new(_value: u64) -> Result<Self, IndexError> {
        todo!("milestone 1: validate positive document ids")
    }

    /// Returns the identifier value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for DocumentId {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!("milestone 3: validate persisted document ids")
    }
}

/// Settings recorded in a version-1 index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSettings {
    /// Lowercased, deduplicated, sorted `.ext` filters that select files.
    pub extensions: Vec<String>,
    /// Inclusive upper bound on a file's byte size; larger files become issues.
    pub max_bytes: u64,
}

impl IndexSettings {
    /// Validates, normalizes, deduplicates, and sorts index settings.
    pub fn new(_extensions: Vec<String>, _max_bytes: u64) -> Result<Self, IndexError> {
        todo!("milestone 1: validate extensions and max-bytes")
    }

    /// Returns whether a portable path has an included extension.
    #[must_use]
    pub fn includes_path(&self, _path: &str) -> bool {
        todo!("milestone 2: match extensions case-insensitively")
    }
}

impl Default for IndexSettings {
    fn default() -> Self {
        Self {
            extensions: vec![".log".into(), ".md".into(), ".txt".into()],
            max_bytes: 1_048_576,
        }
    }
}

/// One normalized term count within a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermCount {
    /// The normalized term as produced by the tokenizer.
    pub term: String,
    /// Occurrences of the term in the document; always positive when persisted.
    pub count: u64,
}

/// One fully indexed document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedDocument {
    /// Contiguous 1-based identifier matching the document's position in order.
    pub id: DocumentId,
    /// Name of the root this document was found under.
    pub root: String,
    /// Portable `/`-joined path relative to the root.
    pub path: String,
    /// Size of the file in bytes at index time.
    pub bytes: u64,
    /// Unique, sorted per-document term counts.
    pub terms: Vec<TermCount>,
}

/// Stable recoverable issue codes recorded in an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

    /// Returns the deterministic report message for this issue category.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::EntryUnreadable => "directory entry could not be read",
            Self::FileUnreadable => "file could not be read",
            Self::FileDisappeared => "file disappeared before it could be read",
            Self::FileTooLarge => "file exceeds the configured byte limit",
            Self::NonUtf8Content => "file content is not valid UTF-8",
            Self::NonUtf8Path => "relative path is not valid UTF-8",
            Self::SymlinkSkipped => "symbolic links are not indexed",
            Self::TokenTooLong => "document contains a token longer than 64 Unicode scalar values",
        }
    }
}

/// One recoverable path issue emitted by indexing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexIssue {
    /// Root name the issue was observed under.
    pub root: String,
    /// Portable path, or `None` only for a `non_utf8_path` entry with no UTF-8 path.
    pub path: Option<String>,
    /// Stable category for the issue.
    pub code: IssueCode,
    /// Human message; must equal `code.message()` when persisted.
    pub message: String,
}

/// Complete versioned index data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexData {
    /// Schema version; only [`crate::INDEX_SCHEMA_VERSION`] is accepted.
    pub schema_version: u64,
    /// Settings the index was built with.
    pub settings: IndexSettings,
    /// Root names in their supplied order, which defines document/issue ordering.
    pub roots: Vec<String>,
    /// Documents sorted by `(root order, path)` with contiguous ids.
    pub documents: Vec<IndexedDocument>,
    /// Recoverable issues sorted by `(root order, path, code, message)`.
    pub issues: Vec<IndexIssue>,
}

impl IndexData {
    /// Revalidates every invariant after loading untrusted JSON.
    pub fn validate(&self) -> Result<(), IndexError> {
        todo!("milestone 3: validate every persisted index invariant")
    }
}

/// Validated search input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchQuery {
    /// Unique, sorted normalized terms combined with logical AND.
    pub terms: Vec<SearchTerm>,
    /// Optional portable path prefix restricting matched documents.
    pub path_prefix: Option<String>,
    /// Maximum matches to return, in `1..=10000`.
    pub limit: usize,
}

impl SearchQuery {
    /// Normalizes terms, validates the optional portable prefix, and bounds limit.
    pub fn new(
        _terms: Vec<String>,
        _path_prefix: Option<String>,
        _limit: usize,
    ) -> Result<Self, IndexError> {
        todo!("milestone 1: validate deterministic query input")
    }

    /// Revalidates a query assembled through its public result-shape fields.
    pub fn validate(&self) -> Result<(), IndexError> {
        todo!("milestone 1: revalidate query invariants")
    }
}

/// Document fields exposed by a search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocumentSummary {
    /// Identifier of the matched document.
    pub id: DocumentId,
    /// Root name the document belongs to.
    pub root: String,
    /// Portable path of the matched document.
    pub path: String,
    /// Size of the document in bytes.
    pub bytes: u64,
}

/// One matching document and its requested term counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchMatch {
    /// Summary of the matched document.
    pub document: DocumentSummary,
    /// Counts for the query terms, in query order.
    pub term_counts: Vec<TermCount>,
}

/// Complete deterministic search response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchResult {
    /// Echo of the executed query.
    pub query: SearchQuery,
    /// Matches in index order, truncated to the query limit.
    pub matches: Vec<SearchMatch>,
}

/// Deterministic index summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexStats {
    /// Schema version of the summarized index.
    pub schema_version: u64,
    /// Number of roots.
    pub roots: usize,
    /// Number of indexed documents.
    pub documents: usize,
    /// Number of recorded issues.
    pub issues: usize,
    /// Count of distinct terms across all documents.
    pub unique_terms: usize,
    /// Sum of indexed document sizes in bytes.
    pub indexed_bytes: u64,
}

impl IndexStats {
    /// Computes summary statistics from a validated index.
    pub fn from_index(_index: &IndexData) -> Result<Self, IndexError> {
        todo!("milestone 3: compute deterministic statistics")
    }
}

/// Converts a host path under `root` into the portable persisted form.
#[must_use]
pub fn portable_relative_path(_root: &Path, _path: &Path) -> Option<String> {
    todo!("milestone 2: convert a contained host path to slash form")
}

/// Checks the persisted portable relative-path grammar.
#[must_use]
pub fn valid_portable_path(_value: &str) -> bool {
    todo!("milestone 1: validate portable relative paths")
}
