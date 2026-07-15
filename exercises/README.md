# 🧩✨ Practice Exercises

Each lesson module has a matching compiler-checked exercise.

Every module folder contains:

- `exercises.rs` — starter functions with `todo!()` and tests that describe the
  required behavior;
- `solutions.rs` — one clear reference implementation with executable checks;
- `README.md` — task descriptions and the exact Cargo commands.

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
