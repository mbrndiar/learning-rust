# Exercises: Module 6 — Errors, Modules, and I/O

Implement:

- parsing a strictly positive integer with a useful error string;
- reading UTF-8 text and trimming surrounding whitespace;
- writing numbered lines to a path;
- propagating I/O errors with `?`.

Run:

```bash
cargo test --example ex-06-errors-io
cargo run --example solution-06-errors-io
```

Tests use a temporary-directory path. Do not convert I/O failure into success or
panic at the file boundary.

## Hints

1. Convert parse errors with `map_err`, then validate zero separately.
2. `fs::read_to_string(path)?` returns owned UTF-8 text or propagates I/O failure.
3. `writeln!` accepts a mutable file and appends the newline.
4. Let the function return `io::Result<()>`; do not catch an error you cannot
   recover from meaningfully.
