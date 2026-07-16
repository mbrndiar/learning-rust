# 🎓✨ Course Lessons

This is the main course content: twelve modules of small, self-contained,
runnable Rust programs. Each module builds on the previous ones.

Modules 1–9 are the core beginner path. Modules 10–12 apply those foundations to
integration, concurrency, and async Rust; take them on a second pass if the first
nine modules and a small project are already a substantial workload.

Run a lesson from the repository root:

```bash
cargo run --example lesson-01-hello-world
```

## 🗺️ Learning modules

1. [`01_basics/`](01_basics/README.md) — programs, bindings, types, strings, functions
2. [`02_control_flow/`](02_control_flow/README.md) — expressions, `match`, loops, ranges
3. [`03_ownership_and_borrowing/`](03_ownership_and_borrowing/README.md) — moves,
   `Copy`, `Clone`, references, slices
4. [`04_structs_enums_and_patterns/`](04_structs_enums_and_patterns/README.md) —
   domain types and exhaustive patterns
5. [`05_collections_iterators_and_closures/`](05_collections_iterators_and_closures/README.md) —
   standard collections and lazy pipelines
6. [`06_errors_modules_and_io/`](06_errors_modules_and_io/README.md) — recoverable
   failure, project structure, paths, files
7. [`07_generics_traits_and_lifetimes/`](07_generics_traits_and_lifetimes/README.md) —
   reusable behavior and reference relationships
8. [`08_testing/`](08_testing/README.md) — unit, integration, and documentation tests
9. [`09_tooling_and_debugging/`](09_tooling_and_debugging/README.md) — the Cargo
   feedback loop and compiler-guided debugging
10. [`10_application_integration/`](10_application_integration/README.md) — JSON,
    validation, TCP, and HTTP boundaries
11. [`11_concurrency/`](11_concurrency/README.md) — threads, channels, and shared state
12. [`12_async_rust/`](12_async_rust/README.md) — futures, Tokio, and bounded tasks

## 🔄 Recommended study loop

1. **Preview:** read the objectives and name unfamiliar terms.
2. **Predict:** write down expected output, type, or ownership behavior.
3. **Experiment:** change one thing and let the compiler test your model.
4. **Practice:** solve the exercise without copying the lesson.
5. **Review:** answer the questions aloud or in writing.
6. **Rebuild:** close the lesson and recreate one example from memory.

If Cargo vocabulary or Rust punctuation is unfamiliar, read the
[`Beginner's Guide`](../docs/BEGINNER_GUIDE.md). If ownership is the sticking
point, use the visual
[`Ownership and Borrowing guide`](../docs/OWNERSHIP_AND_BORROWING.md) before
leaving module 3.

## 🏁 Checkpoints

- After modules 1–2, write a menu-driven unit converter.
- After module 3, explain every move and borrow in that program.
- After modules 4–5, build an in-memory inventory with iterators.
- After modules 6–7, persist domain values behind a trait.
- After modules 8–9, add tests, docs, formatting, and Clippy.
- After module 10, define and validate a JSON or network boundary.
- After modules 11–12, justify whether concurrency improves the design.

These checkpoints are open-ended. Define inputs, outputs, invariants, and failure
behavior before writing code.

After module 12, continue with the [`capstone track`](../capstones/README.md).
The retained [`Task Manager`](../project/task_manager/README.md) is the shortest
complete application to read first; the
[`concept map`](../capstones/MIGRATION.md) connects its architecture to both
capstones.
