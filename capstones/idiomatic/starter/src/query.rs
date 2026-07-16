//! Milestone 1: deterministic in-memory exact-term querying.

use crate::{IndexData, IndexError, SearchQuery, SearchResult};

/// Executes an AND query against validated index data.
pub fn search(_index: &IndexData, _query: SearchQuery) -> Result<SearchResult, IndexError> {
    todo!("milestone 1: query exact normalized terms in index order")
}
