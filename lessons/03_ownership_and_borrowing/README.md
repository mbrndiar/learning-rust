# 🔐 Module 3: Ownership and Borrowing

Ownership lets Rust manage memory and prevent data races without a garbage
collector. It is a compile-time model of who may use a value and for how long.

## 🎯 Learning objectives

After this module, you should be able to explain moves, recognize `Copy` types,
clone intentionally, choose owned versus borrowed parameters, create mutable
borrows, and return slices tied to their input.

## 📦 Ownership and moves

Each value has one owner. Assignment or function calls move non-`Copy` values:

```rust
let first = String::from("owned");
let second = first;
// println!("{first}"); // error: `first` was moved
```

The move prevents two variables from independently freeing the same allocation.
Simple stack-only values such as integers usually implement `Copy`, so assignment
duplicates their bits. `clone()` performs an explicit potentially expensive
duplication.

## 🤝 Borrowing

A reference borrows a value without taking ownership:

```rust
fn length(text: &str) -> usize {
    text.len()
}
```

At a given time, Rust allows either any number of shared references (`&T`) or
one exclusive mutable reference (`&mut T`). References must never outlive their
referent. The compiler often ends a borrow at its final use rather than at the
closing brace (non-lexical lifetimes).

A slice such as `&str` or `&[T]` borrows a contiguous region. Returning a slice
is often better than returning an index because the type preserves the
relationship to the original data.

## 📚 Lessons

- `01_moves_copy_and_clone.rs` — ownership transfer, `Copy`, explicit cloning,
  ownership across function boundaries
- `02_references_and_slices.rs` — shared and mutable borrows, borrow scopes,
  string and array slices

## ▶️ Running

```bash
cargo run --example lesson-03-moves-clone
cargo run --example lesson-03-references-slices
```

Then practice with
[`exercises/03_ownership_and_borrowing/`](../../exercises/03_ownership_and_borrowing/README.md).

## ⚠️ Common mistakes

- Adding `.clone()` until code compiles without understanding the ownership
  transfer.
- Taking `String` or `Vec<T>` when a function only reads `&str` or `&[T]`.
- Holding a mutable borrow longer than necessary.
- Trying to mutate while shared references are still used.
- Returning a reference to a local value that will be dropped.
- Treating borrowing as runtime locking; ordinary references are checked at
  compile time and have no lock overhead.

## ❓ Review questions

1. Why does assigning a `String` move it by default?
2. What promise does the `Copy` trait make?
3. When is `clone()` appropriate?
4. What is the shared-versus-exclusive borrowing rule?
5. Why can a returned slice safely refer to an input but not to a local `String`?
