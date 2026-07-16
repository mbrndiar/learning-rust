//! Public scaffold for the idiomatic concurrent file indexer capstone.
//!
//! Starter and solution packages expose matching boundaries. Milestone
//! operations return [`IndexError::Incomplete`] until implemented.

pub mod build;
pub mod cli;
pub mod domain;
pub mod error;
pub mod query;
pub mod storage;
pub mod tokenization;
pub mod tree;

pub use build::{Cancellation, IndexBuilder};
pub use domain::{
    DocumentId, DocumentSummary, IndexData, IndexIssue, IndexSettings, IndexedDocument, IssueCode,
    RootSpec, SearchMatch, SearchQuery, SearchResult, SearchTerm, TermCount,
};
pub use error::{ErrorCode, IndexError};
pub use storage::IndexStore;
pub use tree::{FileIssue, FileTree, TreeEntry, TreeEntryKind};

/// Version of the JSON index shape defined by the capstone specification.
pub const INDEX_SCHEMA_VERSION: u64 = 1;
