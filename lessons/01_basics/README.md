# 🧱 Module 1: Basics

Rust programs are compiled, statically typed, expression-oriented, and organized
by Cargo. This module introduces the syntax without hiding the type information
that later ownership rules depend on.

## 🎯 Learning objectives

After this module, you should be able to build and run an example, create
immutable and mutable bindings, distinguish common scalar and compound types,
work with `String` and `&str`, and define functions that return expressions.

## 📦 Bindings and types

Bindings are immutable by default:

```rust
let language = "Rust";
let mut attempts = 1;
attempts += 1;
```

`const` names compile-time values and requires an explicit type. Shadowing
creates a new binding and may change type; mutation changes the value of the
same binding and must preserve its type.

Rust's scalar types include integers, floating-point numbers, `bool`, and
Unicode scalar values (`char`). Tuples and fixed-size arrays are compound types.
The compiler usually infers types, but annotations document boundaries and
resolve ambiguity.

## 🖨️ Formatting, macros, and `Debug`

`println!` and `format!` are macros—the `!` is part of their names. `{}` uses a
type's user-facing `Display` formatting. `{:?}` uses developer-oriented `Debug`
formatting and is especially useful for tuples, arrays, options, and structs:

```rust
let point = (3, 4);
println!("{point:?}"); // (3, 4)
```

For your own struct or enum, `#[derive(Debug)]` asks the compiler to generate the
`Debug` implementation:

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}
```

The `#[...]` line is an attribute: metadata that changes how the compiler or a
tool handles the following item. Module 4 uses derives on domain types, and
module 7 explains the traits they implement.

## 📝 `String` and `&str`

`String` owns growable UTF-8 text. `&str` is a borrowed view into valid UTF-8
text, including string literals:

```rust
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
```

Accept `&str` when a function only needs to read text. It works with both string
literals and borrowed `String` values and avoids forcing callers to allocate.
UTF-8 means byte length and character count can differ; ordinary indexing is
therefore not available on strings.

## 📚 Lessons

- `01_hello_world.rs` — program entry point, output macros, formatting
- `02_variables_and_types.rs` — bindings, constants, shadowing, scalar and
  compound types
- `03_strings_and_functions.rs` — owned and borrowed text, functions,
  expressions, statement boundaries

## ▶️ Running

```bash
cargo run --example lesson-01-hello-world
cargo run --example lesson-01-variables-types
cargo run --example lesson-01-strings-functions
```

Then practice with [`exercises/01_basics/`](../../exercises/01_basics/README.md).

## ⚠️ Common mistakes

- Adding `mut` automatically instead of first asking whether state must change.
- Confusing shadowing with mutation.
- Assuming an unsuffixed integer always has the same concrete type.
- Confusing `{}` (`Display`) with `{:?}` (`Debug`) formatting.
- Passing owned `String` when a read-only `&str` is sufficient.
- Adding a semicolon to a function's final expression and accidentally returning
  `()`.
- Treating `String::len()` as a count of human-visible characters.

## ❓ Review questions

1. Why are bindings immutable by default?
2. How do mutation and shadowing differ?
3. What is the difference between `[T; N]` and `(T, U)`?
4. What do the `!` in `println!` and `#[derive(Debug)]` communicate?
5. When should a parameter be `&str` instead of `String`?
6. What does a block return when its final expression has no semicolon?
