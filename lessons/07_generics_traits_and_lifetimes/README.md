# 🧬 Module 7: Generics, Traits, and Lifetimes

Generics abstract over types, traits abstract over behavior, and lifetime
parameters describe relationships between borrowed values. Together they make
reuse explicit without sacrificing static checking.

## 🎯 Learning objectives

After this module, you should be able to write generic functions and structs,
constrain type parameters with traits, implement and use traits, choose static
or dynamic dispatch, and add lifetime annotations when elision cannot express
the relationship.

## 🧰 Generics and trait bounds

Generic code states what varies. Trait bounds state what operations the
implementation requires:

```rust
fn largest<T: PartialOrd>(values: &[T]) -> Option<&T> {
    values.iter().reduce(|left, right| if left >= right { left } else { right })
}
```

Prefer the weakest useful bound. `impl Trait` is concise for a parameter or one
opaque return type; named type parameters express relationships between
multiple positions. `where` clauses improve readability for complex bounds.

Traits can provide default methods. Implementing a trait for a type enables
static dispatch in generic code. `dyn Trait` uses dynamic dispatch and type
erasure when a collection must contain different concrete types.

## ⏳ Lifetimes

Every reference has a lifetime. Most are inferred by elision rules. Explicit
annotations do not extend a value's lifetime; they describe which references
must remain valid together:

```rust
fn longest<'a>(left: &'a str, right: &'a str) -> &'a str;
```

The returned reference cannot outlive the shorter input lifetime. Owned return
values often avoid complex lifetime coupling, but copying data only to avoid
learning lifetimes is not automatically a better design.

## 📚 Lessons

- `01_generics_and_traits.rs` — generic data and functions, bounds, defaults,
  static dispatch
- `02_lifetimes_and_trait_objects.rs` — lifetime relationships, borrowed
  structs, object-safe traits, heterogeneous collections

## ▶️ Running

```bash
cargo run --example lesson-07-generics-traits
cargo run --example lesson-07-lifetimes-trait-objects
```

Then practice with
[`exercises/07_generics_traits_and_lifetimes/`](../../exercises/07_generics_traits_and_lifetimes/README.md).

## ⚠️ Common mistakes

- Adding every familiar trait bound instead of only required behavior.
- Assuming generics imply runtime type checks; Rust monomorphizes most generic
  code.
- Adding lifetime syntax before identifying the actual reference relationship.
- Believing `'static` means a value lives forever in every context.
- Using trait objects when a generic parameter would preserve type information
  and avoid dynamic dispatch.

## ❓ Review questions

1. What does a trait bound permit a generic implementation to do?
2. How do `impl Trait` and a named generic parameter differ?
3. What is static dispatch?
4. What does a lifetime annotation describe rather than change?
5. When is `Box<dyn Trait>` useful?
