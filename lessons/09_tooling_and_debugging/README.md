# 🛠️🔍 Module 9: Tooling and Debugging

Rust's everyday workflow is deliberately integrated. Cargo coordinates targets
and dependencies; rustfmt, Clippy, rustdoc, tests, and compiler diagnostics each
answer a different question.

## 🎯 Learning objectives

After this module, you should be able to navigate Cargo metadata, use a
narrow-to-wide feedback loop, interpret compiler diagnostics, keep CLI parsing
at the application boundary, and distinguish formatting, linting, compilation,
testing, and documentation.

## 🔁 One change, several kinds of feedback

| Tool | Question | Command |
| --- | --- | --- |
| `cargo check` | Does code parse, resolve, and type-check? | `cargo check --workspace --all-targets --locked` |
| rustfmt | Is formatting canonical? | `cargo fmt --all --check` |
| Clippy | Do static patterns suggest bugs or clearer idioms? | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| tests | Does observed behavior match assertions? | `cargo test -p tasks-solution --locked` |
| cargo-llvm-cov | Which code did those tests execute? | `cargo llvm-cov -p tasks-solution --all-targets --summary-only --fail-under-lines 85 --locked` |
| cargo-audit | Does the lockfile contain known vulnerable dependencies? | `cargo audit` |
| rustdoc | Do public examples compile and explain the API? | `cargo test --doc --workspace --locked` |
| Cargo build | Can final artifacts be produced? | `cargo build --workspace --locked` |
| local link check | Do repository-local Markdown links resolve? | `python3 scripts/check-markdown-links.py` |
| GitHub Actions | Does the flow pass in a clean environment? | `.github/workflows/course.yml` |

Passing one row does not imply the others pass. Formatting cannot prove
correctness; types cannot prove requirements; tests and coverage cannot prove
that every requirement or input was considered. See the official
[`cargo-llvm-cov` documentation](https://github.com/taiki-e/cargo-llvm-cov)
for report formats and filtering options.

## 🧭 Read diagnostics systematically

1. Run the smallest command that reproduces the failure.
2. Read the primary error message and code (for example `E0382`).
3. Follow labels showing where a value was moved, borrowed, or expected.
4. Read `note` for the rule and `help` for mechanisms—not guaranteed designs.
5. State the violated invariant in your own words.
6. Make one change and rerun the narrow command.
7. Add a regression test when the problem was behavioral.

Use `rustc --explain E0382` for a longer explanation of an error code.
`dbg!(&value)` temporarily prints an expression with file and line information
and returns the value; remove exploratory diagnostics before committing unless
they are intentional.

## 🔬 A worked borrow diagnostic: E0502

Try this in a disposable package:

```rust,compile_fail
fn main() {
    let mut values = vec![10, 20, 30];
    let first = &values[0];
    values.push(40);
    println!("first={first}");
}
```

The abbreviated diagnostic is:

```text
error[E0502]: cannot borrow `values` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:5
  |
3 |     let first = &values[0];
  |                  ------ immutable borrow occurs here
4 |     values.push(40);
  |     ^^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("first={first}");
  |                     ------- immutable borrow later used here
```

The problem is not that vectors can never be mutated after borrowing. `push`
may reallocate the vector, which would invalidate `first`, and the final line
proves the reference must remain valid across the mutation.

Possible designs:

- use `first` before `push`, allowing its borrow to end;
- copy the integer with `let first = values[0];` because `i32: Copy`;
- restructure the operation so a reference is not held across mutation; or
- use an index only when later lookup has the intended semantics.

Do not solve this by hunting for punctuation. Identify where the immutable
borrow starts, where it is last used, and why mutation overlaps that interval.

## ⌨️ Keep parsing at the boundary

Parse environment or CLI strings once, convert them to domain types, and call
logic that knows nothing about terminal output or process exit codes. This makes
the core callable from tests, another binary, or a network handler.

The standard library exposes raw arguments with `std::env::args`. Production
CLIs commonly use [Clap](https://docs.rs/clap/), demonstrated in the capstone,
for validation, subcommands, and generated help.

Interactive terminal input follows the same boundary rule:

```rust
use std::io;

fn read_age() -> Result<u8, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().parse()?)
}
```

`read_line` appends input including the newline, so trim before parsing. Keep the
`String` and I/O details near the boundary; pass the resulting `u8` into domain
logic.

## 📘 Lessons

- `01_cargo_workflow.rs` — workspace concepts, profiles, lockfiles, feedback
  commands
- `02_diagnostics_and_cli.rs` — pure parsing, typed options, boundary errors,
  debugging workflow

## 🚀 Running

```bash
cargo run --example lesson-09-cargo-workflow
cargo run --example lesson-09-diagnostics-cli -- Ada --shout
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Then practice with
[`exercises/09_tooling_and_debugging/`](../../exercises/09_tooling_and_debugging/README.md).

## 🚧 Common mistakes

- Running a full clean build after every tiny edit instead of `cargo check`.
- Applying a Clippy suggestion without preserving behavior or ownership intent.
- Reading only the last diagnostic line and missing the primary rule.
- Fixing a move error by cloning reflexively.
- Mixing argument parsing, printing, file access, and domain logic in one
  function.
- Committing `target/` or omitting `Cargo.lock` for an application workspace.
- Holding a reference across a mutation without locating its final use.

## 🧠 Review questions

1. How does `cargo check` differ from `cargo build`?
2. What separate feedback do rustfmt and Clippy provide?
3. Why should you explain a diagnostic before accepting its suggestion?
4. What belongs at a CLI boundary?
5. Why is a committed lockfile useful for this repository?
6. Why does terminal input need trimming and typed parsing at the boundary?
