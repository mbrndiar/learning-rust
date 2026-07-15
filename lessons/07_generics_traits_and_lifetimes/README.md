# 🧬⏳ Module 7: Generics, Traits, and Lifetimes

Generics abstract over types, traits abstract over behavior, and lifetime
parameters describe relationships between borrowed values. Together they make
reuse explicit without sacrificing static checking.

## 🎯 Learning objectives

After this module, you should be able to write generic functions and structs,
constrain type parameters with traits, implement and use traits, choose static
or dynamic dispatch, and add lifetime annotations when elision cannot express
the relationship.

## 🪜 Learn the abstractions in layers

Do not treat the module as one large feature:

1. **Generics:** a type varies (`T`), while the algorithm stays the same.
2. **Trait bounds:** the algorithm states which operations it needs from `T`.
3. **Trait implementations:** concrete types promise that behavior.
4. **Trait objects:** different concrete types share one runtime collection.
5. **Lifetimes:** borrowed inputs and outputs state how long references relate.

Master the first three before worrying about `dyn Trait` or explicit lifetime
syntax.

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

| Need | Typical form | Dispatch |
| --- | --- | --- |
| one parameter with required behavior | `item: &impl Summary` | static |
| relationships between generic positions | `fn f<T: Trait>(left: T, right: T)` | static |
| complex bounds | `where T: Trait + Other` | static |
| mixed concrete types in one collection | `Box<dyn Trait>` | dynamic |

Static dispatch preserves concrete type information and is the default.
Dynamic dispatch is useful when heterogeneity matters more than knowing each
concrete type at compile time.

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

Read the signature as a relationship:

```text
left:   |------------------|
right:      |--------|
return:     |--------|       safe overlap of both inputs
```

Start without explicit annotations. Rust commonly infers them when:

- one borrowed input is the obvious source of a borrowed output; or
- a method returns data borrowed from `&self` or `&mut self`.

Add a named lifetime when several input references make the output relationship
ambiguous. `'a` names a constraint; it does not keep a value alive and does not
mean “forever.” The detailed ownership guide provides more examples and a
[signature decision table](../../docs/OWNERSHIP_AND_BORROWING.md).

## 📘 Lessons

- `01_generics_and_traits.rs` — generic data and functions, bounds, defaults,
  static dispatch
- `02_lifetimes_and_trait_objects.rs` — lifetime relationships, borrowed
  structs, object-safe traits, heterogeneous collections

## 🚀 Running

```bash
cargo run --example lesson-07-generics-traits
cargo run --example lesson-07-lifetimes-trait-objects
```

Then practice with
[`exercises/07_generics_traits_and_lifetimes/`](../../exercises/07_generics_traits_and_lifetimes/README.md).

## 🚧 Common mistakes

- Adding every familiar trait bound instead of only required behavior.
- Assuming generics imply runtime type checks; Rust monomorphizes most generic
  code.
- Adding lifetime syntax before identifying the actual reference relationship.
- Believing `'static` means a value lives forever in every context.
- Using trait objects when a generic parameter would preserve type information
  and avoid dynamic dispatch.
- Adding lifetime annotations to owned values such as `String` or `Vec<T>`,
  which already manage their own lifetime.

## 🧠 Review questions

1. What does a trait bound permit a generic implementation to do?
2. How do `impl Trait` and a named generic parameter differ?
3. What is static dispatch?
4. What does a lifetime annotation describe rather than change?
5. When is `Box<dyn Trait>` useful?
6. When can lifetime elision infer a borrowed return value?
