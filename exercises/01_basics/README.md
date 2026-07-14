# Exercises: Module 1 — Basics

Implement in `exercises.rs`:

- `format_profile(name, age)` → `"Ada is 36 years old"`
- `fahrenheit_to_celsius(value)` using the standard conversion formula
- `character_count(text)` as a Unicode character count, not byte length
- `rectangle_area(width, height)` using a final expression

Run:

```bash
cargo test --example ex-01-basics
cargo run --example solution-01-basics
```

Add a test containing non-ASCII text so byte and character counts differ.
