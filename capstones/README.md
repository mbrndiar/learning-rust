# 🏆 Rust capstones

This track contains two deliberately different projects:

- [`comparative/`](comparative/README.md) implements the frozen
  `comparative-kv` contract shared with the other learning repositories.
- [`idiomatic/`](idiomatic/README.md) builds a concurrent file indexer that
  emphasizes Rust ownership, traits, threads, channels, deterministic data, and
  source-preserving errors.

Both projects currently contain **scaffolding only**. Their starter and solution
packages expose matching public boundaries, compile under the workspace lints,
and share a smoke contract. Milestone behavior remains intentionally incomplete.
The existing [`Task Manager`](../project/task_manager/README.md) stays available
as the completed reference project during the migration.

## Learner workflow

1. Read the project `SPEC.md` and README.
2. Work in `starter/`, one milestone at a time.
3. Add or enable tests named `milestone_1`, `milestone_2`, and so on.
4. Compare behavior and design with `solution/` only after attempting the work.
5. Run the narrow package command before widening to workspace validation.

The solution trees are scaffolds, not completed answers yet. A later pilot owns
the implementation and milestone contracts.

## Scaffold checks

```bash
cargo test -p comparative-kv-starter --test smoke --locked
cargo test -p comparative-kv-solution --test smoke --locked
cargo test -p idiomatic-indexer-starter --test smoke --locked
cargo test -p idiomatic-indexer-solution --test smoke --locked
```

Future milestone tests use stable Cargo filters:

```bash
cargo test -p comparative-kv-solution milestone_1 --locked
cargo test -p idiomatic-indexer-solution milestone_1 --locked
```

Final repository gates remain:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --lib --bins --locked
cargo test --doc --workspace --locked
```
