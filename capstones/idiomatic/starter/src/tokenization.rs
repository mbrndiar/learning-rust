//! Milestone 1: deterministic standard-library Unicode tokenization.

/// Maximum normalized token length in Unicode scalar values.
pub const MAX_TERM_CHARS: usize = 64;

/// Result used by the builder to record one long-token issue per document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokenization {
    pub tokens: Vec<String>,
    pub ignored_long_token: bool,
}

/// Tokenizes and normalizes text according to the capstone specification.
#[must_use]
pub fn tokenize(_text: &str) -> Vec<String> {
    todo!("milestone 1: tokenize maximal Unicode alphanumeric runs")
}

/// Tokenizes text while retaining whether any normalized token was too long.
#[must_use]
pub fn tokenize_with_outcome(_text: &str) -> Tokenization {
    todo!("milestone 1: report ignored long tokens")
}
