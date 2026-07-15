# 🧪🧩 Exercises: Module 8 — Testing

This module reverses the usual exercise: the implementation is supplied and the
test bodies contain `todo!()`.

Write tests for:

- ordinary and normalized slug creation;
- empty slug input;
- successful division;
- division by zero with the exact error variant.

Run:

```bash
cargo test --example ex-08-testing
cargo run --example solution-08-testing
```

Make each test fail for the intended reason if you deliberately break the
implementation.

## 💡 Hint ladder

1. Assert the whole returned value, not only one substring.
2. Use `assert_eq!` with `Ok(...)` and the exact `Err(...)` variant.
3. Keep each test focused on one behavior so its failure name is informative.
4. Temporarily break one implementation branch to prove the corresponding test
   catches it, then restore the code.
