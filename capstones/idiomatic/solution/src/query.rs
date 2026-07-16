//! Deterministic in-memory exact-term query boundary.

use crate::{
    DocumentSummary, IndexData, IndexError, SearchMatch, SearchQuery, SearchResult, TermCount,
};

/// Executes an AND query against validated index data.
pub fn search(index: &IndexData, query: SearchQuery) -> Result<SearchResult, IndexError> {
    index.validate()?;
    query.validate()?;
    let mut matches = Vec::new();
    for document in &index.documents {
        if query
            .path_prefix
            .as_ref()
            .is_some_and(|prefix| !document.path.starts_with(prefix))
        {
            continue;
        }

        let mut term_counts = Vec::with_capacity(query.terms.len());
        let mut all_present = true;
        for requested in &query.terms {
            match document
                .terms
                .binary_search_by(|term| term.term.as_str().cmp(requested.as_str()))
            {
                Ok(index) => term_counts.push(TermCount {
                    term: requested.as_str().to_owned(),
                    count: document.terms[index].count,
                }),
                Err(_) => {
                    all_present = false;
                    break;
                }
            }
        }
        if all_present {
            matches.push(SearchMatch {
                document: DocumentSummary {
                    id: document.id,
                    root: document.root.clone(),
                    path: document.path.clone(),
                    bytes: document.bytes,
                },
                term_counts,
            });
            if matches.len() == query.limit {
                break;
            }
        }
    }
    Ok(SearchResult { query, matches })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DocumentId, INDEX_SCHEMA_VERSION, IndexSettings, IndexedDocument, SearchTerm, TermCount,
    };

    #[test]
    fn applies_and_prefix_and_limit_in_index_order() {
        let index = IndexData {
            schema_version: INDEX_SCHEMA_VERSION,
            settings: IndexSettings::default(),
            roots: vec!["fixture".into()],
            documents: vec![IndexedDocument {
                id: DocumentId::new(1).expect("id"),
                root: "fixture".into(),
                path: "docs/readme.md".into(),
                bytes: 4,
                terms: vec![
                    TermCount {
                        term: "rust".into(),
                        count: 2,
                    },
                    TermCount {
                        term: "safe".into(),
                        count: 1,
                    },
                ],
            }],
            issues: Vec::new(),
        };
        let result = search(
            &index,
            SearchQuery {
                terms: vec![
                    SearchTerm::parse("rust").expect("term"),
                    SearchTerm::parse("safe").expect("term"),
                ],
                path_prefix: Some("docs".into()),
                limit: 1,
            },
        )
        .expect("search");
        assert_eq!(result.matches.len(), 1);
    }
}
