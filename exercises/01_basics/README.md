# 🌱🧩 Exercises: Module 1 — Basics

Implement in `exercises.rs`:

- `format_profile(name, age)` → `"Ada is 36 years old"`
- `fahrenheit_to_celsius(value)` using the standard conversion formula
- `character_count(text)` as a Unicode character count, not byte length
- `rectangle_area(width, height)` with lossless widening to `u64`
- `capped_progress(current, completed)` with saturating arithmetic

Run:

```bash
cargo test --example ex-01-basics
cargo run --example solution-01-basics
```

Add a test containing non-ASCII text so byte and character counts differ.

## 💡 Hint ladder

1. `format!` returns a `String`; `println!` only writes output.
2. Use floating-point literals in the temperature formula.
3. UTF-8 byte length is `.len()`, while `.chars().count()` answers this task.
4. Convert each `u32` with `u64::from` before multiplying.
5. `saturating_add` stops at the type's maximum instead of wrapping.
