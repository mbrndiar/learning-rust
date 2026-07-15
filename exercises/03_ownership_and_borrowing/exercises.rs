//! Exercises for module 3.

pub fn first_word(_text: &str) -> &str {
    todo!("return a slice ending at the first whitespace")
}

#[allow(clippy::ptr_arg)] // The exercise intentionally grows an owned String.
pub fn append_period(_text: &mut String) {
    todo!("append one period only when missing")
}

pub fn total_byte_length(_values: &[String]) -> usize {
    todo!("borrow each string and sum its byte length")
}

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
