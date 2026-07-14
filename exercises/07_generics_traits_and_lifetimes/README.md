# Exercises: Module 7 — Generics, Traits, and Lifetimes

Implement:

- generic `largest` over a borrowed slice;
- a `Label` trait for two concrete types;
- rendering heterogeneous `&dyn Label` values;
- `longest` with the lifetime relationship required by the return value.

Run:

```bash
cargo test --example ex-07-traits-lifetimes
cargo run --example solution-07-traits-lifetimes
```

Use the weakest useful bounds and do not allocate inside `longest`.
