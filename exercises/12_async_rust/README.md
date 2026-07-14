# Exercises: Module 12 — Async Rust

Implement:

- `delayed_double` with Tokio's non-blocking sleep;
- `double_all` by spawning all values concurrently;
- awaiting every task and returning the results in ascending order.

Run:

```bash
cargo test --example ex-12-async
cargo run --example solution-12-async
```

Do not use `std::thread::sleep` and do not discard a join error.
