//! Lesson 5.1: vectors, strings, maps, sets, and entry APIs.
//!
//! `Vec<T>` is a growable array and `VecDeque<T>` a double-ended queue.
//! `HashMap`/`BTreeMap` associate keys with values, with `BTreeMap` keeping keys
//! sorted. The `entry` API inserts-or-updates in a single lookup, and `get`
//! returns an `Option`, so out-of-range access is handled instead of panicking.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

fn word_counts(text: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for word in text.split_whitespace().map(str::to_lowercase) {
        // `entry` looks the key up once. `or_insert(0)` returns a `&mut usize` to
        // the existing or freshly inserted count, which `*` then increments.
        *counts.entry(word).or_insert(0) += 1;
    }
    counts // a BTreeMap iterates in sorted key order
}

fn main() {
    let mut scores = vec![80, 95, 70];
    scores.push(88);
    // `first_mut` yields `Option<&mut i32>`; the borrow lets us edit in place.
    if let Some(first) = scores.first_mut() {
        *first += 1;
    }
    // `get` returns `Option`, so an out-of-range index is `None`, never a panic.
    println!("scores={scores:?}, safe index={:?}", scores.get(10));

    let mut queue = VecDeque::from(["parse", "compile"]);
    queue.push_back("test");
    // `pop_front` returns `None` once the deque empties, which ends the loop.
    while let Some(step) = queue.pop_front() {
        println!("next step: {step}");
    }

    let mut capitals = HashMap::new();
    capitals.insert("Czechia", "Prague");
    capitals.insert("Japan", "Tokyo");
    println!("capital of Czechia={:?}", capitals.get("Czechia"));

    // A `HashSet` keeps only distinct values; the duplicate "rust" collapses.
    let tags: HashSet<_> = ["rust", "safe", "fast", "rust"].into_iter().collect();
    println!(
        "{} unique tags; contains safe={}",
        tags.len(),
        tags.contains("safe")
    );

    let counts = word_counts("Rust makes systems programming safer Rust makes ownership visible");
    println!("ordered counts={counts:?}");

    let mut message = String::from("UTF-8");
    message.push_str(" text");
    // As in module 1, byte length and character count differ for non-ASCII text.
    println!(
        "{message:?}: {} bytes, {} chars",
        message.len(),
        message.chars().count()
    );
}
