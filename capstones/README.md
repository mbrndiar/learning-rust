# 🏆 Rust capstones

This track contains two deliberately different projects:

- [`comparative/`](comparative/README.md) implements the frozen
  `comparative-kv` contract shared with the other learning repositories.
- [`idiomatic/`](idiomatic/README.md) builds a concurrent file indexer that
  emphasizes Rust ownership, traits, threads, channels, deterministic data, and
  source-preserving errors.

Both solutions are complete and fixture-driven. Each starter is a guided,
compilable milestone scaffold with matching public boundaries and ignored
contract groups. The durable predecessor-to-capstone concept map is in
[`MIGRATION.md`](MIGRATION.md).

## Prerequisites

Complete Modules 1–13 and the
[`Task applied project`](../projects/tasks/README.md) first. Module 10 directly
prepares the comparative SQLite store; Modules 11–12 prepare the idiomatic
indexer's concurrency choices; Module 13 prepares the JSON and client/server
boundaries used across applied work.

## Learner workflow

1. Read the project `SPEC.md` and README.
2. Work in `starter/`, one milestone at a time.
3. Run a test named `milestone_1`, `milestone_2`, and so on with
   `-- --ignored`.
4. Compare behavior and design with `solution/` only after attempting the work.
5. Run the narrow package command before widening to workspace validation.

Starter package tests succeed by default because unfinished milestone wrappers
are ignored; this verifies that the scaffold and smoke boundary compile, not that
the starter conforms. A filtered ignored milestone is intentionally red until
its behavior is implemented. Remove only that milestone wrapper's `#[ignore]`
after it passes. The comparative subprocess helper entry points stay ignored
because the contracts launch them explicitly.

## Package checks

```bash
cargo test -p comparative-kv-starter --locked
cargo test -p comparative-kv-solution --locked
cargo test -p idiomatic-indexer-starter --locked
cargo test -p idiomatic-indexer-solution --locked
```

Milestone tests use stable Cargo filters. The starter commands below are
expected to fail before the selected milestone is complete:

```bash
cargo test -p comparative-kv-solution milestone_1 --locked
cargo test -p comparative-kv-starter milestone_1 --locked -- --ignored
cargo test -p idiomatic-indexer-solution milestone_1 --locked
cargo test -p idiomatic-indexer-starter milestone_1 --locked -- --ignored
```

## Repository gates

```bash
python3 scripts/check-markdown-links.py
cargo metadata --format-version 1 --locked --no-deps > /dev/null
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --lib --bins --locked
cargo test -p tasks-contracts --locked
cargo test -p tasks-starter --locked
cargo test -p tasks-solution --locked
cargo test -p comparative-kv-solution --locked
cargo test -p idiomatic-indexer-solution --locked
cargo test --doc --workspace --locked
cargo doc --workspace --no-deps --locked
cargo audit
cargo llvm-cov -p tasks-solution --all-targets --summary-only --fail-under-lines 85 --locked
cargo llvm-cov -p comparative-kv-solution --all-targets --summary-only --locked
cargo llvm-cov -p idiomatic-indexer-solution --all-targets --summary-only --locked
```

CI enforces 85% line coverage for the completed Task solution and reports
coverage for the two complete capstone solutions. It deliberately does not score
the incomplete starters or apply numeric thresholds to the capstones.
