# 🧬🧩 Exercises: Module 7 — Generics, Traits, and Lifetimes

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

## 💡 Hint ladder

1. `Iterator::reduce` returns `None` for an empty iterator.
2. The generic algorithm only compares values, so `PartialOrd` is enough.
3. A slice of `&dyn Label` can contain references to different concrete types.
4. Both possible `longest` return values come from the inputs, so one named
   lifetime must connect them to the output.
