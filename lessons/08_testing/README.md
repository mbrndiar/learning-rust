# 🧪✅ Module 8: Testing

Tests are ordinary Rust functions compiled in a special configuration. Good
tests document behavior, isolate resources, and make failures specific enough to
diagnose.

## 🎯 Learning objectives

After this module, you should be able to write unit and integration tests,
assert success and failure behavior, organize test-only helpers, design
deterministic dependencies, and understand what test coverage cannot prove.

## 🔬 Test forms

- Unit tests usually live beside implementation code in `#[cfg(test)] mod tests`.
  They may inspect private implementation details, though behavior-focused tests
  are less brittle.
- Integration tests live in `tests/` and use the library through its public API.
- Documentation tests compile examples in `///` and `//!` comments and verify
  that public usage stays accurate.

Useful macros include `assert!`, `assert_eq!`, `assert_ne!`, and `matches!`.
Tests may return `Result<(), E>` to use `?`. Use `#[should_panic]` only when panic
is the intended contract; recoverable invalid input should normally return
`Result`.

## 🎛️ Deterministic design

Time, randomness, environment variables, files, and networks make tests flaky
when accessed globally. Put those effects behind a parameter or trait and inject
a controlled implementation. Test pure transformations directly and keep I/O at
thin boundaries.

Each test should arrange one scenario, act once, and assert the important
observable result. Test normal cases, boundaries, and expected failures without
depending on execution order.

## 📘 Lessons

- `01_unit_tests.rs` — test modules, result assertions, private helpers
- `02_test_design.rs` — dependency injection, deterministic clocks, table-style
  cases, behavior-focused checks

## 🚀 Running

```bash
cargo test --example lesson-08-unit-tests
cargo test --example lesson-08-test-design
cargo test --workspace --lib --bins
cargo test --doc --workspace
```

Then practice with [`exercises/08_testing/`](../../exercises/08_testing/README.md).

## 🚧 Common mistakes

- Testing only the happy path.
- Sharing mutable global state between tests.
- Depending on wall-clock time, real user files, or external services.
- Asserting internal implementation details rather than public behavior.
- Marking a broad test `#[should_panic]` without checking why it panicked.
- Treating a passing suite or high coverage as proof that requirements are
  complete.

## 🧠 Review questions

1. When can unit tests access private items?
2. What does `#[cfg(test)]` change?
3. Why might a test return `Result<(), E>`?
4. How does dependency injection improve determinism?
5. Which boundary and failure cases should accompany a normal-case test?
