# 🧩✨ Practice Exercises

Each lesson module has a matching compiler-checked exercise.

Every module folder contains:

- `exercises.rs` — starter functions with `todo!()` and tests that describe the
  required behavior;
- `solutions.rs` — one clear reference implementation with executable checks;
- `README.md` — task descriptions and the exact Cargo commands.

## 🗺️ Exercise modules

1. [`01_basics/`](01_basics/README.md)
2. [`02_control_flow/`](02_control_flow/README.md)
3. [`03_ownership_and_borrowing/`](03_ownership_and_borrowing/README.md)
4. [`04_structs_enums_and_patterns/`](04_structs_enums_and_patterns/README.md)
5. [`05_collections_iterators_and_closures/`](05_collections_iterators_and_closures/README.md)
6. [`06_errors_modules_and_io/`](06_errors_modules_and_io/README.md)
7. [`07_generics_traits_and_lifetimes/`](07_generics_traits_and_lifetimes/README.md)
8. [`08_testing/`](08_testing/README.md)
9. [`09_tooling_and_debugging/`](09_tooling_and_debugging/README.md)
10. [`10_sql_and_sqlite/`](10_sql_and_sqlite/README.md)
11. [`11_concurrency/`](11_concurrency/README.md)
12. [`12_async_rust/`](12_async_rust/README.md)
13. [`13_rest_apis_and_http_clients/`](13_rest_apis_and_http_clients/README.md)

## 🚀 How to work through an exercise

1. Read the matching lesson module.
2. Open `exercises/<module>/exercises.rs`.
3. Replace each `todo!()` while preserving the function signatures.
4. Run the module's test target, for example:

   ```bash
   cargo test --example ex-03-ownership
   ```

5. Add at least one boundary test of your own.
6. Compare with `solutions.rs` only after a genuine attempt.

## ⚙️ Why starters compile

Rust's `todo!()` macro has the never type (`!`), which can coerce to any return
type. Cargo can therefore parse, type-check, format, and lint the entire course
before every exercise is solved. Running a test that reaches `todo!()` panics,
which makes unfinished behavior impossible to mistake for success.

## 🕵️ Using solutions well

Compare more than output:

- Does each function take ownership only when it needs to?
- Could an input be a slice or `&str` instead of an owned collection?
- Is invalid input represented by `Option`, `Result`, or a domain enum?
- Does iterator code remain readable?
- Are error messages and tests precise at boundaries?

A different implementation may be equally idiomatic when it communicates the
same contract and ownership decisions clearly.
