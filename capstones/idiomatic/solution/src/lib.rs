//! Concurrent deterministic file indexer and search library.
//!
//! The solution uses strict validated values, injected tree/store seams, bounded
//! standard-library workers, stable ordering, and atomic versioned JSON output.

pub mod build;
pub mod cli;
pub mod domain;
pub mod error;
pub mod query;
pub mod storage;
pub mod tokenization;
pub mod tree;

pub use build::{Cancellation, CancellationToken, IndexBuilder};
pub use domain::{
    DocumentId, DocumentSummary, IndexData, IndexIssue, IndexSettings, IndexStats, IndexedDocument,
    IssueCode, RootSpec, SearchMatch, SearchQuery, SearchResult, SearchTerm, TermCount,
    portable_relative_path, valid_portable_path, validate_roots,
};
pub use error::{ErrorCode, IndexError};
pub use storage::{IndexStore, JsonFileIndexStore};
pub use tree::{FileIssue, FileTree, StdFileTree, TreeEntry, TreeEntryKind};

/// Version of the JSON index shape defined by the capstone specification.
pub const INDEX_SCHEMA_VERSION: u64 = 1;
