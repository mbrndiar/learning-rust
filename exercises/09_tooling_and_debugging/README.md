# Exercises: Module 9 — Tooling and Debugging

Implement a small parsing boundary:

- parse `--verbose`;
- accept exactly one optional input path;
- reject unknown flags and extra positional values;
- keep `build_summary` independent of argument strings.

Run:

```bash
cargo test --example ex-09-tooling
cargo run --example solution-09-tooling
```

After tests pass, run `cargo fmt --all` and Clippy. Read any diagnostic fully
before changing the code.

## Hints

1. Track the optional path and verbose flag in separate local variables.
2. Match known flags before a guard that rejects other `-`-prefixed values.
3. A second positional value should return immediately with `Err`.
4. Parsing should build `Options`; presentation belongs in `build_summary`.
