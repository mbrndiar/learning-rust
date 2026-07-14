//! Lesson 5.1: vectors, strings, maps, sets, and entry APIs.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

fn word_counts(text: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for word in text.split_whitespace().map(str::to_lowercase) {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

fn main() {
    let mut scores = vec![80, 95, 70];
    scores.push(88);
    if let Some(first) = scores.first_mut() {
        *first += 1;
    }
    println!("scores={scores:?}, safe index={:?}", scores.get(10));

    let mut queue = VecDeque::from(["parse", "compile"]);
    queue.push_back("test");
    while let Some(step) = queue.pop_front() {
        println!("next step: {step}");
    }

    let mut capitals = HashMap::new();
    capitals.insert("Czechia", "Prague");
    capitals.insert("Japan", "Tokyo");
    println!("capital of Czechia={:?}", capitals.get("Czechia"));

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
    println!(
        "{message:?}: {} bytes, {} chars",
        message.len(),
        message.chars().count()
    );
}
