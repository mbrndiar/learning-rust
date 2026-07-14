# 🧩 Module 4: Structs, Enums, and Patterns

Rust's algebraic data types let the type system describe valid domain states.
Methods attach behavior without inheritance, and exhaustive patterns force code
to consider every variant.

## 🎯 Learning objectives

After this module, you should be able to define structs and enums, implement
methods and associated functions, use `Option<T>` instead of sentinel values,
destructure values, and choose `match`, `if let`, or `let else`.

## 🏗️ Structs and methods

Named-field structs model records. Tuple structs create distinct types without
field names; unit structs carry no data. An `impl` block defines associated
functions and methods:

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}
```

Use `&self` to read, `&mut self` to modify, and `self` to consume the value.
Constructors named `new` are conventions, not language magic.

## 🎭 Enums and patterns

Each enum variant may carry different data. `Option<T>` is either `Some(T)` or
`None`, making absence explicit. A `match` must cover every possible variant.

Use `if let` for one interesting pattern and an uninteresting remainder. Use
`let PATTERN = value else { ... };` when a function should exit unless one
pattern matches. Prefer a full `match` when several cases matter.

## 📚 Lessons

- `01_structs_and_methods.rs` — domain records, constructors, methods, update
  syntax, tuple structs
- `02_enums_option_and_patterns.rs` — data-carrying variants, `Option`,
  destructuring, guards, concise pattern forms

## ▶️ Running

```bash
cargo run --example lesson-04-structs-methods
cargo run --example lesson-04-enums-patterns
```

Then practice with
[`exercises/04_structs_enums_and_patterns/`](../../exercises/04_structs_enums_and_patterns/README.md).

## ⚠️ Common mistakes

- Exposing fields publicly before deciding which invariants methods should
  preserve.
- Taking ownership of `self` when a method only needs to read.
- Encoding absence with `0`, `-1`, or an empty string instead of `Option`.
- Using `_` to silence a newly added enum variant that deserves behavior.
- Replacing a clear exhaustive `match` with nested `if let` chains.

## ❓ Review questions

1. How do a method and an associated function differ?
2. What do `self`, `&self`, and `&mut self` communicate?
3. Why is `Option<T>` safer than a sentinel value?
4. What happens to matches when an enum gains a variant?
5. When is `let else` clearer than `match`?
