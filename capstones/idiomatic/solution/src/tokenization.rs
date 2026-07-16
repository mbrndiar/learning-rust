//! Deterministic standard-library Unicode tokenization.

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
pub(crate) fn normalize_search_term(value: &str) -> Option<String> {
    let outcome = tokenize_with_outcome(value);
    if outcome.ignored_long_token || outcome.tokens.len() != 1 {
        None
    } else {
        outcome.tokens.into_iter().next()
    }
}

/// Checks the persisted normalized-term invariant.
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
