# 🌊🧩 Exercises: Module 12 — Async Rust

Implement:

- `delayed_double` with Tokio's non-blocking sleep;
- `double_all` with at most `max_in_flight` spawned tasks at once;
- awaiting every task and returning the results in ascending order.

Run:

```bash
cargo test --example ex-12-async
cargo run --example solution-12-async
```

Do not use `std::thread::sleep` and do not discard a join error.

## 💡 Hint ladder

1. `tokio::time::sleep(delay).await` suspends without blocking the executor
   thread.
2. Normalize `max_in_flight` with `.max(1)`, then seed a `JoinSet` from the input
   iterator.
3. Each time `join_next().await` returns a result, spawn at most one replacement
   from the iterator. This keeps work overlapping without creating every task.
4. A joined result contains `Result<T, JoinError>`; `?` preserves task failure.
5. Sort only after all successful results have been collected.
