//! Deterministic in-memory query boundary.

use crate::{IndexData, IndexError, SearchQuery, SearchResult};

/// Executes an exact-term query against validated index data.
pub fn search(_index: &IndexData, _query: SearchQuery) -> Result<SearchResult, IndexError> {
    Err(IndexError::incomplete("index querying"))
}
