//! Milestone 1: deterministic in-memory exact-term querying.
//!
//! Search must be pure and side-effect free: read a validated [`IndexData`] and
//! return matches in the index's own `(root order, path)` order, then truncate to
//! the limit. Multiple terms use logical AND, and matching is exact on normalized
//! terms — no substring, prefix, ranking, or fuzzy behavior.

use crate::{IndexData, IndexError, SearchQuery, SearchResult};

/// Executes an AND query against validated index data.
pub fn search(_index: &IndexData, _query: SearchQuery) -> Result<SearchResult, IndexError> {
    todo!("milestone 1: query exact normalized terms in index order")
}
