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

## Hints

1. Trim the title before checking `is_empty` and before storing it.
2. `Self { ... }` constructs the current type inside its `impl`.
3. Completing an already-complete task is not an error; compare old and new
   state through the returned `bool`.
4. A `match` over `Priority` should list every variant explicitly.
