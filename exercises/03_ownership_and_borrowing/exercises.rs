//! Exercises for module 3: ownership, borrowing, and slices.
//!
//! Implement each function by replacing its `todo!()` body, then run the example
//! tests. Do not change any signature; note which functions borrow their input
//! and which take ownership.

/// Return the slice of `text` up to the first whitespace character.
///
/// The result borrows from `text`, so it is valid only while `text` lives. When
/// `text` has no whitespace, the whole string is returned.
pub fn first_word(_text: &str) -> &str {
    todo!("return a slice ending at the first whitespace")
}

/// Append a single `.` to `text`, but only when it does not already end with one.
///
/// Mutates the caller's `String` in place through the exclusive `&mut` borrow.
#[allow(clippy::ptr_arg)] // The exercise intentionally grows an owned String.
pub fn append_period(_text: &mut String) {
    todo!("append one period only when missing")
}

/// Sum the byte lengths of every string in `values`.
///
/// Borrows the slice read-only; the caller keeps ownership and may reuse it.
pub fn total_byte_length(_values: &[String]) -> usize {
    todo!("borrow each string and sum its byte length")
}

/// Consume `text` and return an uppercased owned `String`.
///
/// Takes ownership by value, so the caller's original binding is moved away.
pub fn into_uppercase(_text: String) -> String {
    todo!("consume the input and return uppercase text")
}

fn main() {
    println!("Run `cargo test --example ex-03-ownership` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_a_borrowed_word() {
        let sentence = String::from("borrow precisely");
        assert_eq!(first_word(&sentence), "borrow");
        assert_eq!(first_word("single"), "single");
    }

    #[test]
    fn mutates_without_duplicate_punctuation() {
        let mut text = String::from("Rust");
        append_period(&mut text);
        append_period(&mut text);
        assert_eq!(text, "Rust.");
    }

    #[test]
    fn reads_a_slice_without_consuming_it() {
        let values = vec![String::from("ab"), String::from("rust")];
        assert_eq!(total_byte_length(&values), 6);
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn consumes_and_replaces_owned_text() {
        assert_eq!(into_uppercase(String::from("move me")), "MOVE ME");
    }
}
