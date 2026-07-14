# Exercises: Module 4 — Structs, Enums, and Patterns

Complete the `Task` methods and `Priority` behavior:

- validate non-empty task titles in `Task::new`;
- mark a task complete and report whether state changed;
- return a display label for every priority;
- describe `Option<Task>` without sentinel values.

Run:

```bash
cargo test --example ex-04-domain-types
cargo run --example solution-04-domain-types
```

Keep fields private and preserve the invariant through methods.
