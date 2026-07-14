# Exercises: Module 11 — Concurrency

Implement:

- `parallel_sum` by partitioning owned work across threads;
- `worker_messages` with multiple channel producers;
- deterministic result ordering after concurrent execution.

Run:

```bash
cargo test --example ex-11-concurrency
cargo run --example solution-11-concurrency
```

Join every thread and drop the original sender before collecting messages.
