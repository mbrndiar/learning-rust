# 🔐🧩 Exercises: Module 3 — Ownership and Borrowing

Implement signatures that deliberately exercise ownership:

- borrow text to find its first word;
- borrow a mutable string to add punctuation;
- borrow a slice of owned strings to total their byte lengths;
- consume a string and return its uppercase replacement.

Run:

```bash
cargo test --example ex-03-ownership
cargo run --example solution-03-ownership
```

Do not clone inside these functions. The supplied signatures already express the
required ownership.

## 💡 Hint ladder

1. `str::char_indices` yields each character with its safe UTF-8 byte boundary.
2. A mutable reference can call the same `String` methods as its owner.
3. `iter().map(...).sum()` reads every string without consuming the vector.
4. The final function already owns its input, so returning a replacement does
   not require preserving the original binding.
