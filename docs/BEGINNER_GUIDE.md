# 🌱🧭 Beginner's Guide to This Course

This page is the map to read before module 1. It explains what the repository
contains, how Rust's tools fit together, which symbols will look unfamiliar,
and how to learn from compiler errors without getting stuck.

## 👋 Who this course is for

No previous Rust knowledge is required. Familiarity with variables, functions,
conditions, and loops is helpful, but the lessons define Rust-specific terms as
they appear. If programming itself is new, move slowly through modules 1 and 2,
type the examples yourself, and repeat the exercises before continuing.

The complete course has three stages:

| Stage | Modules | Goal |
| --- | --- | --- |
| Foundations | 1–4 | Read basic Rust and understand ownership and domain types |
| Productive Rust | 5–9 | Use collections, errors, traits, tests, Cargo, and diagnostics |
| Applied and advanced | 10–12 | Cross JSON/network boundaries and add concurrency/async |

Modules 10–12 are valuable, but they are not required before building ordinary
synchronous command-line programs. It is reasonable to finish module 9, build a
small project, then return for a second pass.

## ⏱️ Your first 30 minutes

1. Follow [`SETUP.md`](SETUP.md) through running the first lesson.
2. Read [`lessons/01_basics/README.md`](../lessons/01_basics/README.md).
3. Open `lessons/01_basics/01_hello_world.rs` and find `fn main`.
4. Predict the output, then run:

   ```bash
   cargo run --example lesson-01-hello-world
   ```

5. Change `"Rust"` to your name and run it again.
6. Restore or keep the experiment, then run the next lesson.
7. After all module 1 lessons, open
   `exercises/01_basics/exercises.rs`, replace one `todo!()`, and run:

   ```bash
   cargo test --example ex-01-basics
   ```

The first failing test is useful feedback, not a failed course attempt.

## 🗺️ Repository map

```text
learning-rust/
├── Cargo.toml       workspace, dependencies, and teaching targets
├── Cargo.lock       exact dependency versions selected by Cargo
├── lessons/         runnable explanations grouped into modules
├── exercises/       starter tasks and reference solutions
├── capstones/       comparative and idiomatic starter/solution packages
├── docs/            setup and deeper learning guides
├── CHEATSHEET.md    syntax and command reference
└── scripts/         repository-wide course runner
```

Rust and Cargo use several related words:

| Word | Meaning in this repository |
| --- | --- |
| package | one `Cargo.toml` describing one or more build targets |
| crate | one Rust compilation unit: a library or executable |
| target | a particular library, binary, test, or example Cargo can build |
| example | a named executable teaching target selected with `--example` |
| workspace | several packages operated on together from the root |
| dependency | another crate declared in `Cargo.toml` |

The lesson files live in module directories rather than Cargo's usual
`examples/` directory, so the root `Cargo.toml` lists every example explicitly.
Your own first project will normally use the simpler generated layout shown in
[`SETUP.md`](SETUP.md).

The root manifest also defines five workspace packages. A command without
`--workspace`, `-p`, or `--example` targets the root course package by default.
Use `-p comparative-kv-solution`, `-p idiomatic-indexer-solution`, or another
package name for one application; use `--workspace` only when you intend to check
every member.

## ⚙️ What happens when you run a lesson

For:

```bash
cargo run --example lesson-03-moves-clone
```

Cargo:

1. reads `Cargo.toml` and `Cargo.lock`;
2. finds the example target named `lesson-03-moves-clone`;
3. compiles changed code and required dependencies;
4. stores generated artifacts under `target/`; and
5. runs the resulting executable only if compilation succeeds.

`cargo check` stops after analysis and is usually faster because it does not
produce the final executable. `cargo test` builds a test harness and runs
functions marked `#[test]`.

## 🔣 Rust punctuation legend

Rust uses punctuation to make type and ownership information visible:

| Syntax | Read it as |
| --- | --- |
| `let name = value;` | bind `name` to `value` |
| `name: Type` | `name` has this type |
| `fn f(...) -> T` | function returning `T` |
| `&T` | shared borrow of a `T` |
| `&mut T` | exclusive mutable borrow of a `T` |
| `String::from(...)` | associated function in the `String` namespace |
| `value.method()` | call a method on `value` |
| `println!(...)` | invoke a macro; `!` distinguishes macros from functions |
| `condition?` | return early if this `Result`/`Option` represents failure/absence |
| `pattern => expression` | one `match` arm |
| `T` in `Vec<T>` | a generic type parameter |
| `'a` | a named lifetime relationship |
| `dyn Trait` | a value using dynamic trait dispatch |
| `#[derive(Debug)]` | an attribute asking the compiler to generate behavior |

You do not need to memorize the table first. Return to it when a symbol appears
in a lesson.

## 📖 How to read a lesson

Use the same order every time:

1. Read the module objectives and mental model in its `README.md`.
2. Open the first `.rs` file and identify inputs, owned values, borrows, and
   returned values.
3. Predict output before running it.
4. Run the exact command from the module.
5. Change one thing only: a value, type annotation, branch, borrow, or method.
6. If compilation fails, read the diagnostic before undoing the change.
7. Explain the result in one sentence.
8. Complete the matching exercise and add one boundary test.

The `.rs` files are concise demonstrations; the surrounding README explains why
the code is written that way. Read both surfaces together.

## 🤝 The compiler is a teaching partner

A diagnostic usually contains:

```text
error[E....]: primary description
 --> path:line:column
  |
  | code with labels
  | ^^^ explanation of this location
  |
  = note: the rule or additional context
  = help: a possible language mechanism
```

Use this process:

1. Read the primary `error[...]` line.
2. Find the first labeled line in your code.
3. Follow labels such as “value moved here” and “borrow later used here.”
4. State the rule in plain language.
5. Decide what the function should own or borrow.
6. Only then consider a `help` suggestion.

Compiler help is syntactically informed, not aware of your design. For example,
cloning can satisfy ownership but may be unnecessary if the function should
borrow. Module 3 walks through a moved-value error, and module 9 walks through
an overlapping-borrow error.

## 💡 A non-spoiling hint ladder

When an exercise is difficult, use help in this order:

1. Re-read the function signature and translate each `&`, `&mut`, `Option`, or
   `Result` into words.
2. Read the test name and use one concrete input manually.
3. Search the matching lesson for the required method or pattern.
4. Run the single exercise and read the first failure.
5. Read the exercise README's hints.
6. Ask the compiler about an error code with `rustc --explain E0382`.
7. Look at only the relevant function in `solutions.rs`, close it, and recreate
   the idea without copying.

## 🌱 Healthy beginner defaults

- Prefer immutable bindings until a value truly changes.
- Borrow with `&str` or `&[T]` when a function only reads.
- Return `Option` for expected absence and `Result` for explainable failure.
- Prefer `match` when several enum states matter.
- Start with a loop when an iterator chain would be difficult to explain.
- Do not add `.clone()`, `unwrap()`, `'static`, `Arc`, or `Mutex` only to silence
  the compiler. First identify the ownership or failure contract.
- Keep sequential code until concurrency solves a measured or structural need.

## ✅ When to move on

Continue when you can:

- predict the main example's behavior;
- explain the module's central type signatures;
- solve most of the exercise without the reference solution; and
- answer the review questions in your own words.

Perfect recall is not required. Ownership and lifetimes usually become clearer
through repeated use across later modules. If module 3 feels difficult, read
[`OWNERSHIP_AND_BORROWING.md`](OWNERSHIP_AND_BORROWING.md), complete its small
experiments, and then continue.

After module 12, choose a project from the
[`capstone track`](../capstones/README.md). The
[`migration concept map`](../capstones/MIGRATION.md) explains how design habits
from the removed predecessor transfer to the comparative and idiomatic contracts.
