# 🦀✨ learning-rust

A complete, hands-on introduction to modern Rust for independent learners. The
course combines written explanations, small runnable programs, compiler-guided
exercises with solutions, review questions, a tested capstone project, and a
syntax reference. No previous Rust experience is assumed.

If Rust syntax, Cargo terminology, or compiler diagnostics are completely new,
begin with the [`Beginner's Guide`](docs/BEGINNER_GUIDE.md). It provides a
first-30-minutes path and a symbol-by-symbol legend before module 1.

## 🎯 What you will learn

By the end of the course, you will be able to:

- create, build, run, format, lint, test, and document Cargo projects;
- choose appropriate scalar, compound, collection, and domain types;
- control program flow with expressions, loops, pattern matching, and iterators;
- reason about ownership, moves, cloning, borrowing, slices, and lifetimes;
- model valid states with structs, enums, `Option`, and `Result`;
- write reusable code with functions, closures, generics, traits, and modules;
- handle recoverable errors without panics or hidden failure;
- read and write UTF-8 files and serialize validated data with Serde;
- design useful unit, integration, and documentation tests;
- use threads, channels, locks, `Arc`, futures, and Tokio tasks safely; and
- design, implement, test, and extend an idiomatic command-line application.

## 🧰 Requirements

- Rust 1.85+ with Cargo (the first stable release supporting Rust 2024)
- Git for cloning the repository
- An internet connection the first time Cargo downloads the few crates used by
  the integration lessons and capstone project

New to Rust or setting up for the first time? See
[`docs/SETUP.md`](docs/SETUP.md) for installing Rust with `rustup`, choosing an
editor, and understanding the toolchain.

## 🚀 How to run a lesson

Every lesson is a Cargo example. From the repository root:

```bash
cargo run --example lesson-01-hello-world
```

Do not only run the files. For each module:

1. Read its `README.md`, including the examples and common mistakes.
2. Predict a lesson's output or compiler behavior before running it.
3. Run it, then change one value or type and observe what changes.
4. Answer the module's review questions without looking back.
5. Complete its exercise before reading `solutions.rs`.
6. Revisit anything you cannot explain in your own words.

The modules build on one another. Beginners should follow them in order.
Modules 1–9 form the core path. Modules 10–12 are applied/advanced material and
can be deferred until after a first small synchronous project.

## 🔄 Developer feedback loop

Rust's compiler is part of the learning loop. Start with the narrowest command
that exercises your change, then widen the feedback:

```bash
cargo run --example lesson-03-references-slices
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib --bins
cargo test -p task-manager
cargo test --doc --workspace
```

Run all lessons and reference solutions with:

```bash
bash scripts/run-course.sh
```

Module 9 explains what each command checks and how the same flow maps to
[GitHub Actions](https://docs.github.com/en/actions).

## 📐 Conventions used in this course

- Terminal commands are marked `bash`; Rust code is marked `rust`.
- Code that intentionally does not compile is shown in documentation, not in
  runnable lesson files. The explanation identifies the compiler rule involved.
- `...` in a code sample means omitted code unless the surrounding text says it
  is a range.
- Examples prefer explicit, readable code over clever shortcuts.
- `unwrap()` and `expect()` appear only where the lesson has already proved the
  invariant or a short-lived example cannot recover meaningfully.
- Public application boundaries return typed errors instead of silently using
  defaults.

## 🧩 Practice exercises

Each module has a matching folder under [`exercises/`](exercises/README.md).
Starter functions contain `todo!()` so the file still compiles while clearly
marking unfinished work. Run one exercise's tests with:

```bash
cargo test --example ex-01-basics
```

After making a genuine attempt, compare your code with the matching
`solutions.rs` or run the reference implementation:

```bash
cargo run --example solution-01-basics
```

## 🏆 Capstone project

The [`Task Manager`](project/task_manager/README.md) combines domain modeling,
traits, dependency injection, Serde, atomic file persistence, Clap, typed
errors, and tests in one command-line application:

```bash
cargo run -p task-manager -- add "Learn ownership"
cargo run -p task-manager -- list
```

The supplied implementation is a readable starting point. Build features from
the extension list one at a time and add a test for each behavior change.

## ⚡ Cheat sheet

[`CHEATSHEET.md`](CHEATSHEET.md) is a compact glossary, syntax reference, and
command guide for use after the course.

## 🗺️ Course outline

1. **[Basics](lessons/01_basics/)** — programs, bindings, scalar and compound
   types, strings, functions, expressions
2. **[Control Flow](lessons/02_control_flow/)** — `if`, `match`, loops, ranges,
   labels, early exit
3. **[Ownership and Borrowing](lessons/03_ownership_and_borrowing/)** — moves,
   `Copy`, `Clone`, references, mutable borrows, slices
4. **[Structs, Enums, and Patterns](lessons/04_structs_enums_and_patterns/)** —
   methods, associated functions, `Option`, exhaustive matching
5. **[Collections, Iterators, and Closures](lessons/05_collections_iterators_and_closures/)** —
   `Vec`, `String`, maps, sets, iterator adapters, closures
6. **[Errors, Modules, and I/O](lessons/06_errors_modules_and_io/)** — `Result`,
   `?`, custom errors, modules, paths, files
7. **[Generics, Traits, and Lifetimes](lessons/07_generics_traits_and_lifetimes/)** —
   generic code, trait bounds, trait objects, lifetime relationships
8. **[Testing](lessons/08_testing/)** — unit tests, test organization, failure
   cases, deterministic design
9. **[Tooling and Debugging](lessons/09_tooling_and_debugging/)** — Cargo,
   rustfmt, Clippy, rustdoc, compiler diagnostics, CLI boundaries
10. **[Application Integration](lessons/10_application_integration/)** — Serde,
    JSON validation, TCP and small HTTP boundaries
11. **[Concurrency](lessons/11_concurrency/)** — threads, channels, `Arc`,
    `Mutex`, lock scope
12. **[Async Rust](lessons/12_async_rust/)** — futures, `.await`, Tokio tasks,
    bounded concurrency

For ownership's deeper visual model, compiler labs, and signature decision
tables, use [`docs/OWNERSHIP_AND_BORROWING.md`](docs/OWNERSHIP_AND_BORROWING.md)
alongside module 3.

## 🛟 Getting help from the material

Read compiler diagnostics from the top: the primary message states the violated
rule, labels point to relevant values, and `help` often suggests a valid
mechanism. Do not apply a suggestion blindly—explain why it satisfies ownership,
type, or lifetime constraints first.

Solutions are examples, not the only correct answers. Compare behavior,
readability, ownership choices, failure handling, and tests rather than requiring
identical code.

## 🧭 Course boundaries

This course aims to make a beginner independently productive with safe,
idiomatic Rust. Specialized work such as embedded systems, WebAssembly, advanced
async services, unsafe abstractions, procedural macros, and performance tuning
requires focused study after these foundations. The final section of
[`CHEATSHEET.md`](CHEATSHEET.md) points to authoritative next steps.
