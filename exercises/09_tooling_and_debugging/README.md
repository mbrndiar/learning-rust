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
