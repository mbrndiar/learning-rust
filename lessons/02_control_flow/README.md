# 🚦🔀 Module 2: Control Flow

Rust control-flow constructs are expressions: they can produce values while
deciding which path executes.

## 🎯 Learning objectives

After this module, you should be able to choose between `if` and `match`, write
exhaustive patterns, iterate with loops and ranges, return values from `loop`,
and use labels only when nested control flow needs them.

## 🌿 Branching expressions

`if` conditions must be `bool`; Rust does not implicitly treat numbers or
collections as truthy or falsey. Every branch used as a value must produce the
same type:

```rust
let label = if score >= 60 { "pass" } else { "retry" };
```

`match` compares one value against exhaustive patterns. Put specific patterns
before catch-alls such as `_`. Guards add a condition without making the match
non-exhaustive.

## 🔁 Loops

- `for value in iterable` is the default for traversing a sequence.
- `while condition` repeats while a boolean condition holds.
- `loop` repeats until `break`; `break value` makes the loop an expression.
- labels such as `'search:` let `break` or `continue` target an outer loop.

Ranges are half-open (`0..3` gives 0, 1, 2) or inclusive (`0..=3` includes 3).
Prefer iterating over values or references instead of manual indexes unless the
index itself matters.

## 📘 Lessons

- `01_conditionals_and_match.rs` — conditional expressions, pattern matching,
  guards, destructuring
- `02_loops_and_ranges.rs` — `for`, `while`, value-returning `loop`, labels

## 🚀 Running

```bash
cargo run --example lesson-02-conditionals-match
cargo run --example lesson-02-loops-ranges
```

Then practice with
[`exercises/02_control_flow/`](../../exercises/02_control_flow/README.md).

## 🚧 Common mistakes

- Writing `if number` instead of a boolean comparison.
- Returning incompatible types from `if` branches.
- Forgetting that `0..end` excludes `end`.
- Adding `_ => ...` too early and hiding a domain case that should be explicit.
- Indexing a collection when direct iteration is clearer and safer.
- Using nested labeled loops when the logic should be extracted into a function.

## 🧠 Review questions

1. Why must both branches of a value-producing `if` have one type?
2. What does exhaustive matching protect you from?
3. How do `0..5` and `0..=5` differ?
4. When can `loop` produce a value?
5. Why is direct iteration usually preferable to indexing?
