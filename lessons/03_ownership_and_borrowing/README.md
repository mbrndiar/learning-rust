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

A useful model for `String` is a small owned handle pointing to a UTF-8 buffer:

```text
binding               owned allocation
+------------+         +-------------------+
| pointer ---+-------->| o | w | n | e | d |
| length     |         +-------------------+
| capacity   |
+------------+
```

Moving the `String` transfers responsibility for that allocation. It does not
duplicate the text. See the visual
[`Ownership and Borrowing guide`](../../docs/OWNERSHIP_AND_BORROWING.md) for a
step-by-step model of moves, borrows, slices, and lifetime relationships.

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

## 🧪 Break it on purpose: E0382

In a disposable `cargo new ownership-lab` project, compile:

```rust,compile_fail
fn main() {
    let first = String::from("owned");
    let second = first;
    println!("new owner: {second}");
    println!("old owner: {first}");
}
```

The stable compiler reports a diagnostic similar to this abbreviated output:

```text
error[E0382]: borrow of moved value: `first`
 --> src/main.rs:5:26
  |
2 |     let first = String::from("owned");
  |         ----- move occurs because `first` has type `String`
3 |     let second = first;
  |                  ----- value moved here
...
5 |     println!("old owner: {first}");
  |                          ^^^^^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
```

Read it in order:

1. **Primary rule:** code borrowed a value after ownership moved.
2. **First label:** `String` is not `Copy`.
3. **Second label:** the assignment transferred ownership.
4. **Final label:** the old binding was used after the transfer.
5. **Help:** cloning is one mechanism, not automatically the right design.

Choose a fix from the intended data flow:

- If only `second` should own the string, remove the final use of `first`.
- If `second` only reads, use `let second = &first;`.
- If both must independently own text, use `let second = first.clone();`.
- If a function consumed too much, change its parameter from `String` to `&str`.

Run `rustc --explain E0382` for the compiler's longer explanation, then restore
the compiling version before continuing.

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
- Applying the compiler's clone suggestion before deciding whether ownership or
  borrowing matches the design.

## ❓ Review questions

1. Why does assigning a `String` move it by default?
2. What promise does the `Copy` trait make?
3. When is `clone()` appropriate?
4. What is the shared-versus-exclusive borrowing rule?
5. Why can a returned slice safely refer to an input but not to a local `String`?
6. What three design choices can resolve a moved-value error?
