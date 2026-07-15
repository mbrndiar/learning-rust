//! Exercises for module 5: collections, iterators, and closures.
//!
//! Implement each `todo!()` body, then run the example tests. Do not change any
//! signature.

use std::collections::BTreeMap;

/// Return the values of `values` sorted ascending with duplicates removed.
pub fn unique_sorted(_values: &[i32]) -> Vec<i32> {
    todo!("collect through an ordered set")
}

/// Count how often each word appears, case-insensitively.
///
/// Words are lowercased into owned `String` keys; the `BTreeMap` keeps them in
/// sorted order.
pub fn word_frequencies(_words: &[&str]) -> BTreeMap<String, usize> {
    todo!("count owned lowercase words")
}

/// Return the squares of the even values in `values`, preserving input order.
pub fn even_squares(_values: &[i32]) -> Vec<i32> {
    todo!("filter even values and square them")
}

/// Group `words` by their first character.
///
/// Empty words are skipped. Within each group, words keep their input order; the
/// `BTreeMap` orders the groups by their first character.
pub fn group_by_first_character(_words: &[&str]) -> BTreeMap<char, Vec<String>> {
    todo!("skip empty words and preserve input order within each group")
}

fn main() {
    println!("Run `cargo test --example ex-05-collections` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_in_sorted_order() {
        assert_eq!(unique_sorted(&[3, 1, 2, 1, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn counts_normalized_words() {
        assert_eq!(
            word_frequencies(&["Rust", "safe", "rust"]),
            BTreeMap::from([(String::from("rust"), 2), (String::from("safe"), 1)])
        );
    }

    #[test]
    fn transforms_with_iterators() {
        assert_eq!(even_squares(&[1, 2, 3, 4]), vec![4, 16]);
    }

    #[test]
    fn groups_non_empty_words() {
        assert_eq!(
            group_by_first_character(&["apple", "", "apricot", "banana"]),
            BTreeMap::from([
                ('a', vec![String::from("apple"), String::from("apricot")]),
                ('b', vec![String::from("banana")]),
            ])
        );
    }
}
