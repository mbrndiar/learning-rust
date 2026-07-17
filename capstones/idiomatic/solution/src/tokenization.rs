//! Deterministic standard-library Unicode tokenization.
//!
//! One algorithm normalizes text for both indexing and search so the two can never
//! disagree. A token is a maximal run of `char::is_alphanumeric()` characters,
//! lowercased with `char::to_lowercase()`; everything else (punctuation, symbols,
//! whitespace, `_`, `-`) separates tokens. Length is measured in Unicode scalar
//! values *after* case expansion, because one input character can lowercase into
//! several scalars (for example `İ` becomes `i` + U+0307).

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
pub fn tokenize(text: &str) -> Vec<String> {
    tokenize_with_outcome(text).tokens
}

/// Tokenizes text while retaining whether any normalized token was too long.
#[must_use]
pub fn tokenize_with_outcome(text: &str) -> Tokenization {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_chars = 0_usize;
    let mut ignored_long_token = false;

    let finish = |current: &mut String,
                  current_chars: &mut usize,
                  tokens: &mut Vec<String>,
                  ignored: &mut bool| {
        if current.is_empty() {
            return;
        }
        // Length is counted in scalars, not bytes: an over-length run is dropped
        // and flagged rather than truncated, so partial tokens never enter the index.
        if *current_chars <= MAX_TERM_CHARS {
            tokens.push(std::mem::take(current));
        } else {
            current.clear();
            *ignored = true;
        }
        *current_chars = 0;
    };

    for character in text.chars() {
        if character.is_alphanumeric() {
            // Case folding can expand one character into several scalars; count each.
            for lowered in character.to_lowercase() {
                current.push(lowered);
                current_chars += 1;
            }
        } else {
            finish(
                &mut current,
                &mut current_chars,
                &mut tokens,
                &mut ignored_long_token,
            );
        }
    }
    // Flush the final run, since the loop only finishes a token on a separator.
    finish(
        &mut current,
        &mut current_chars,
        &mut tokens,
        &mut ignored_long_token,
    );

    Tokenization {
        tokens,
        ignored_long_token,
    }
}

/// Returns one normalized term when the input contains exactly one valid token.
///
/// This is how search terms are validated: input that tokenizes to zero tokens,
/// more than one token, or a single over-length token is rejected by returning
/// `None`, guaranteeing a query term matches the same rule as an indexed term.
pub(crate) fn normalize_search_term(value: &str) -> Option<String> {
    let outcome = tokenize_with_outcome(value);
    if outcome.ignored_long_token || outcome.tokens.len() != 1 {
        None
    } else {
        outcome.tokens.into_iter().next()
    }
}

/// Checks the persisted normalized-term invariant.
///
/// Used when reloading an index to confirm a stored term is what the tokenizer
/// would have produced: non-empty, within the scalar limit, already lowercase, and
/// composed only of alphanumerics — plus the one combining mark U+0307 that dotless
/// `i` case folding legitimately emits, hence the explicit allowance below.
pub(crate) fn is_normalized_term(value: &str) -> bool {
    let count = value.chars().count();
    if count == 0 || count > MAX_TERM_CHARS {
        return false;
    }
    if value
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        != value
    {
        return false;
    }

    let mut previous = None;
    value.chars().all(|character| {
        // U+0307 is permitted only immediately after `i`, the output of folding `İ`.
        let valid =
            character.is_alphanumeric() || (character == '\u{307}' && previous == Some('i'));
        previous = Some(character);
        valid
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_separators_case_expansion_and_limits() {
        assert_eq!(
            tokenize("Rust_safe-42! İ"),
            vec!["rust", "safe", "42", "i\u{307}"]
        );
        let long = "A".repeat(MAX_TERM_CHARS + 1);
        let outcome = tokenize_with_outcome(&format!("{long} ok {long}"));
        assert_eq!(outcome.tokens, vec!["ok"]);
        assert!(outcome.ignored_long_token);
    }
}
