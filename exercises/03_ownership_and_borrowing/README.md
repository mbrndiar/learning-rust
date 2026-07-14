# Exercises: Module 3 — Ownership and Borrowing

Implement signatures that deliberately exercise ownership:

- borrow text to find its first word;
- borrow a mutable string to add punctuation;
- borrow a slice of owned strings to total their byte lengths;
- consume a string and return its uppercase replacement.

Run:

```bash
cargo test --example ex-03-ownership
cargo run --example solution-03-ownership
```

Do not clone inside these functions. The supplied signatures already express the
required ownership.
