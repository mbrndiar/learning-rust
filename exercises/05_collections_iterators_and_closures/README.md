# Exercises: Module 5 — Collections, Iterators, and Closures

Implement:

- sorted unique values with a set;
- deterministic word frequencies with a `BTreeMap`;
- even squares with an iterator pipeline;
- grouping words by first character.

Run:

```bash
cargo test --example ex-05-collections
cargo run --example solution-05-collections
```

Avoid indexing strings. Decide how empty words should be handled and preserve the
supplied contract.

## Hints

1. `BTreeSet` deduplicates while preserving sorted iteration.
2. `entry(...).or_insert(0)` gives one mutable count to update.
3. `filter` receives a reference to each iterator item; inspect the exact item
   type when dereferencing feels confusing.
4. `word.chars().next()` represents an empty word as `None`.
