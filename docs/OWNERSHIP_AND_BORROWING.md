# Ownership and Borrowing: A Visual Guide

Ownership is Rust's central memory-safety model. The rules can feel restrictive
when learned as slogans, so this guide follows concrete values through moves,
borrows, mutation, slices, and returned references.

## 1. Start with responsibility, not memory addresses

For every value, ask:

1. Who is responsible for eventually cleaning it up?
2. Does this operation need to read, mutate, store, or consume it?
3. How long must any reference remain usable?

Rust answers these questions in types and function signatures.

## 2. Owned values are cleaned up once

```rust
{
    let text = String::from("Rust");
    println!("{text}");
} // `text` leaves scope; its String allocation is dropped here
```

Conceptually, a `String` contains small bookkeeping values and owns a separate
UTF-8 buffer:

```text
stack-like local state             heap allocation
+----------------------+           +-------------------+
| pointer -------------+---------->| R | u | s | t |
| length: 4            |           +-------------------+
| capacity: 4          |
+----------------------+
```

The exact machine layout is an implementation detail. The important point is
that one `String` owns the buffer and frees it when dropped.

## 3. A move transfers responsibility

```rust
let first = String::from("Rust");
let second = first;
println!("{second}");
```

After the assignment:

```text
first:  no longer usable
second: owns the String buffer
```

Rust may copy the small bookkeeping fields, but it does not duplicate the text
buffer. Disabling `first` prevents two variables from freeing the same buffer.

Integers and some other small types implement `Copy`:

```rust
let first = 42;
let second = first;
println!("{first} {second}"); // both remain usable
```

`Copy` means ordinary assignment duplicates the entire usable value cheaply and
without custom cleanup. A type owning a `String` generally cannot be `Copy`.

## 4. `clone()` duplicates intentionally

```rust
let first = String::from("Rust");
let second = first.clone();
println!("{first} {second}");
```

Now two independent strings own two buffers. Cloning is correct when the program
needs independent owned values. It is not the default fix for every move error:
if another function only needs to read, borrow instead.

## 5. Function signatures state ownership

```rust
fn consume(text: String) {
    println!("{text}");
}

fn inspect(text: &str) {
    println!("{text}");
}

fn extend(text: &mut String) {
    text.push('!');
}
```

| Signature | Caller gives | Function may | After the call |
| --- | --- | --- | --- |
| `text: String` | ownership | store, mutate, or drop it | caller cannot use original |
| `text: &str` | shared borrow | read text | caller keeps ownership |
| `text: &mut String` | exclusive borrow | read and mutate | caller regains use after borrow |

Choose the least ownership a function needs. Accepting `&str` is often more
flexible than `&String` because it accepts literals, string slices, and borrowed
`String` values.

## 6. Shared and exclusive access

Rust's borrowing rule is:

- any number of shared references (`&T`), or
- exactly one exclusive mutable reference (`&mut T`)

for the same data while those references are in use.

Multiple readers are safe:

```rust
let text = String::from("Rust");
let first = &text;
let second = &text;
println!("{first} {second}");
```

One writer is safe:

```rust
let mut text = String::from("Rust");
let writer = &mut text;
writer.push('!');
println!("{text}");
```

An active reader and writer would make the reader observe changing or
reallocated data, so Rust rejects overlapping incompatible borrows.

## 7. Borrows end after their final use

```rust
let mut text = String::from("Rust");

let shared = &text;
println!("{shared}"); // final use of `shared`

text.push('!');       // accepted: shared borrow has ended
```

This is a non-lexical lifetime: the borrow can end before the closing brace.
When a borrow error occurs, locate the reference's final use, not only where it
was created.

## 8. Slices borrow part of a value

```rust
fn first_word(text: &str) -> &str {
    let end = text.find(char::is_whitespace).unwrap_or(text.len());
    &text[..end]
}
```

The returned `&str` points into the caller's text:

```text
input:  "borrow precisely"
         ^^^^^^
output:  slice of the first six bytes
```

The function does not allocate another `String`. The caller cannot mutate the
original `String` in a way that may reallocate it while the slice is still used.

Array and vector slices use the same idea:

```rust
fn first_two(values: &[i32]) -> &[i32] {
    &values[..values.len().min(2)]
}
```

## 9. Lifetimes describe a returned-reference relationship

Most lifetime information is inferred:

```rust
fn first_word(text: &str) -> &str;
```

One borrowed input makes it clear that the returned reference comes from that
input. With two possible sources, name the relationship:

```rust
fn longest<'a>(left: &'a str, right: &'a str) -> &'a str;
```

Read it as:

> The returned reference is valid only for an overlap in which both input
> references are valid.

```text
left:   |-------------------|
right:      |---------|
return:     |---------|       cannot outlive the shorter input
```

`'a` does not keep either string alive and does not mean all three references
have identical real-world lifetimes. It constrains the returned reference to a
safe common relationship.

## 10. A reference cannot outlive local data

This is invalid:

```rust,compile_fail
fn invalid<'a>() -> &'a str {
    let text = String::from("temporary");
    &text
}
```

The named `'a` claims the function can return a reference valid for a lifetime
chosen by its caller, but `text` is dropped when the function returns. The
compiler reports `E0515: cannot return reference to local variable`. Without the
explicit lifetime, it first reports `E0106` because there is no borrowed input
from which an output lifetime could be inferred. Neither annotation can make
local data live longer. Return the owned value instead:

```rust
fn valid() -> String {
    String::from("owned by the caller")
}
```

Return owned data when the function creates it and the caller must keep it.
Return a reference when the result is a view into caller-owned input.

## 11. A practical signature decision table

| Requirement | Starting signature |
| --- | --- |
| read borrowed text | `fn f(text: &str)` |
| read borrowed elements | `fn f(values: &[T])` |
| modify caller-owned value | `fn f(value: &mut T)` |
| keep or consume the argument | `fn f(value: T)` |
| create a new value | `fn f(...) -> T` |
| return a view into one input | `fn f(input: &T) -> &U` |
| return a view from one of several inputs | explicit lifetime relationship |
| express expected absence | `Option<T>` |
| explain recoverable failure | `Result<T, E>` |

These are starting points, not rigid laws. Let the intended data flow choose the
signature before using clones to satisfy an accidental signature.

## 12. Progressive experiments

Use a disposable project so intentionally broken code does not stop the course
workspace from compiling:

```bash
cargo new ownership-lab
cd ownership-lab
```

Try these in order:

1. Move a `String`, use only the new owner, and compile.
2. Use the old binding after the move and read `E0382`.
3. Replace the consuming operation with `&str`.
4. Create two shared references and use both.
5. Create one mutable reference and update the value.
6. Keep a shared reference in use after trying to mutate and read `E0502`.
7. Move the reference's final use before the mutation.
8. Return a slice from an input.
9. Try the explicit-lifetime local-reference example from section 10, read
   `E0515`, and explain why the annotation cannot keep the `String` alive.

Delete the disposable directory when finished. The worked diagnostics in
[module 3](../lessons/03_ownership_and_borrowing/README.md) and
[module 9](../lessons/09_tooling_and_debugging/README.md) show how to interpret
the failures.

## 13. Self-check

Before moving on, explain:

1. Why moving a `String` does not deep-copy its text.
2. Why `Copy` types remain usable after assignment.
3. When cloning represents a real requirement.
4. Why shared references and a mutable reference cannot overlap.
5. Why a slice cannot safely outlive its source.
6. What a lifetime annotation describes rather than changes.
