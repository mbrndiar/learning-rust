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

## Hints

1. Normalize zero workers to at least one, then cap workers at the number of
   values.
2. `slice::chunks` needs a nonzero chunk size; `div_ceil` can calculate it.
3. Move an owned chunk into each thread and sum joined partial results.
4. Clone the sender for each worker, drop the original, collect, then sort.
