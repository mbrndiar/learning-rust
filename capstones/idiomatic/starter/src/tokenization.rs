//! Milestone 1: deterministic standard-library Unicode tokenization.
//!
//! One algorithm must normalize text for both indexing and search so the two can
//! never disagree. A token is a maximal run of `char::is_alphanumeric()` characters,
//! lowercased with `char::to_lowercase()`; everything else separates tokens. Length
//! is measured in Unicode scalar values *after* case expansion, because one input
//! character can lowercase into several scalars (for example `İ` becomes `i` +
//! U+0307). Implement both functions to that single, shared rule.

/// Maximum normalized token length in Unicode scalar values.
pub const MAX_TERM_CHARS: usize = 64;

/// Result used by the builder to record one long-token issue per document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokenization {
    /// Normalized tokens in source order, each within [`MAX_TERM_CHARS`].
    pub tokens: Vec<String>,
    /// Set when at least one over-length run was dropped from `tokens`.
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
