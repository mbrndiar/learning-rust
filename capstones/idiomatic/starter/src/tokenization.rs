//! Unicode tokenization boundary.

use crate::IndexError;

/// Tokenizes and normalizes text according to the capstone specification.
pub fn tokenize(_text: &str) -> Result<Vec<String>, IndexError> {
    Err(IndexError::incomplete("tokenization"))
}
