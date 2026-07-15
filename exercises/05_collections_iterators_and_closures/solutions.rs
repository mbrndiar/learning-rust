//! Reference solutions for module 5.

use std::collections::{BTreeMap, BTreeSet};

fn unique_sorted(values: &[i32]) -> Vec<i32> {
    // A BTreeSet both removes duplicates and yields its elements in sorted order.
    values
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn word_frequencies(words: &[&str]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for word in words {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

fn even_squares(values: &[i32]) -> Vec<i32> {
    values
        .iter()
        .copied()
        .filter(|value| value % 2 == 0)
        .map(|value| value * value)
        .collect()
}

fn group_by_first_character(words: &[&str]) -> BTreeMap<char, Vec<String>> {
    let mut groups: BTreeMap<char, Vec<String>> = BTreeMap::new();
    for word in words {
        // `chars().next()` is `None` for an empty word, so `if let` skips it.
        if let Some(first) = word.chars().next() {
            // `or_default` inserts an empty Vec the first time a key appears.
            groups.entry(first).or_default().push((*word).to_owned());
        }
    }
    groups
}

fn main() {
    assert_eq!(unique_sorted(&[3, 1, 2, 1]), vec![1, 2, 3]);
    assert_eq!(
        word_frequencies(&["Rust", "rust"]),
        BTreeMap::from([(String::from("rust"), 2)])
    );
    assert_eq!(even_squares(&[1, 2, 3, 4]), vec![4, 16]);
    assert_eq!(
        group_by_first_character(&["apple", "", "apricot"]),
        BTreeMap::from([('a', vec![String::from("apple"), String::from("apricot")])])
    );
    println!("Module 5 solutions passed.");
}
