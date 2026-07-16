# 🦀 Idiomatic capstone: concurrent file indexer

The [normative specification](SPEC.md) is implemented by a complete solution and
a matching guided starter. The capstone indexes explicitly named roots without
following symlinks, tokenizes strict UTF-8 with standard-library Unicode rules,
publishes a deterministic versioned JSON index atomically, and provides exact
AND search plus statistics.

## Layout and learning path

```text
idiomatic/
├── SPEC.md
├── starter/    # matching public API and scoped milestone todo!() bodies
├── solution/   # complete bounded-thread implementation
└── tests/
    ├── contracts/   # five groups compiled against both packages
    └── fixtures/    # offline tree, expected reports, and corrupt indexes
```

Both packages expose `domain`, `tree`, `tokenization`, `build`, `storage`,
`query`, and `cli`. The public `FileTree` and `IndexStore` traits make failures
deterministic in tests. `IndexBuilder` owns a positive worker bound and a
cloneable cancellation token; every started worker is joined before return.

Work through the starter's ignored groups in order:

```bash
cargo test -p idiomatic-indexer-starter milestone_1 --locked -- --ignored
cargo test -p idiomatic-indexer-starter milestone_2 --locked -- --ignored
```

The starter keeps signatures, types, docs, Serde shapes, Clap commands, and
error categories aligned with the solution. Only milestone behavior is left as
an intentional `todo!()`.

## Run the solution

```bash
cargo run -p idiomatic-indexer-solution -- \
  index --index index.json --root notes=./notes

cargo run -p idiomatic-indexer-solution -- \
  search --index index.json --term rust --format text

cargo run -p idiomatic-indexer-solution -- \
  stats --index index.json --format json
```

Roots use `NAME=PATH`. Defaults index `.log`, `.md`, and `.txt` files up to
1 MiB. Recoverable path/file issues are stored in the index; invalid roots,
corrupt indexes, worker failures, publication failures, and cancellation use
typed fatal errors and stable exits.

## Tests and quality gates

```bash
cargo test -p idiomatic-indexer-solution milestone_1 --locked
cargo test -p idiomatic-indexer-solution --locked
cargo test -p idiomatic-indexer-starter --locked

cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --lib --bins --locked
cargo test --doc --workspace --locked
cargo llvm-cov -p idiomatic-indexer-solution --all-targets --summary-only --locked
```

Fixtures cover nested and hidden files, empty and Unicode text, oversized and
non-UTF-8 content, symlinks, fake unreadable/disappearing/non-UTF-8 paths,
corrupt index invariants, reverse worker completion, cancellation, panic
containment, deterministic text/JSON reports, and repeatable single/multi-worker
index bytes. Tests scan only their explicit temporary roots and use no network,
database, regex, sleeps, or external data.
