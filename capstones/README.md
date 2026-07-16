# 🏆 Rust capstones

This track contains two deliberately different projects:

- [`comparative/`](comparative/README.md) implements the frozen
  `comparative-kv` contract shared with the other learning repositories.
- [`idiomatic/`](idiomatic/README.md) builds a concurrent file indexer that
  emphasizes Rust ownership, traits, threads, channels, deterministic data, and
  source-preserving errors.

The comparative solution is complete and its starter remains a guided,
compileable milestone scaffold. The idiomatic project still contains scaffolding
only and is owned by a separate implementation pilot. The existing
[`Task Manager`](../project/task_manager/README.md) remains another completed
reference application.

## Learner workflow

1. Read the project `SPEC.md` and README.
2. Work in `starter/`, one milestone at a time.
3. Run or enable tests named `milestone_1`, `milestone_2`, and so on.
4. Compare behavior and design with `solution/` only after attempting the work.
5. Run the narrow package command before widening to workspace validation.

The comparative starter's milestone tests are ignored by default so the
workspace stays green without pretending the starter conforms. Run a selected
group with `-- --ignored` while implementing it.

## Scaffold checks

```bash
cargo test -p comparative-kv-starter --locked
cargo test -p comparative-kv-solution --locked
cargo test -p idiomatic-indexer-starter --test smoke --locked
cargo test -p idiomatic-indexer-solution --test smoke --locked
```

Milestone tests use stable Cargo filters:

```bash
cargo test -p comparative-kv-solution milestone_1 --locked
cargo test -p comparative-kv-starter milestone_1 --locked -- --ignored
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
