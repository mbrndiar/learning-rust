# 🚦🧩 Exercises: Module 2 — Control Flow

Implement:

- `number_kind` using branching or `match`
- `fizz_buzz` for one number
- `sum_until_limit`, stopping before the sum would exceed the limit
- `first_multiple`, returning the first value divisible by a divisor

Run:

```bash
cargo test --example ex-02-control-flow
cargo run --example solution-02-control-flow
```

Consider zero, empty slices, and values divisible by both 3 and 5.

## 💡 Hint ladder

1. Check the combined FizzBuzz case before either individual case.
2. `checked_add` distinguishes arithmetic overflow from an ordinary limit.
3. Return early when a zero divisor would make `% divisor` invalid.
4. `iter().copied().find(...)` can return an owned `u32` from a borrowed slice.
