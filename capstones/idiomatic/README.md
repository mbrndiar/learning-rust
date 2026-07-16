# 🦀 Idiomatic capstone: concurrent file indexer

The [Rust-specific specification](SPEC.md) defines a deterministic, versioned
file index plus `index`, `search`, and `stats` commands. The project is designed
to exercise validated newtypes, borrowing, trait seams, strict UTF-8 I/O,
source-preserving errors, bounded `std::thread` workers, cancellation, stable
ordering, Serde, and publish-after-complete replacement.

## Scaffold layout

```text
idiomatic/
├── SPEC.md
├── starter/    # package: idiomatic-indexer-starter
├── solution/   # package: idiomatic-indexer-solution
└── tests/      # contract code compiled against both packages
```

Both packages expose the same public modules: `domain`, `tree`, `tokenization`,
`build`, `storage`, `query`, and `cli`. The required `FileTree` and `IndexStore`
capabilities are injectable. `IndexBuilder` accepts a positive worker count and
a cloneable `Cancellation` implementation.

The public types and CLI shape are present, but milestone operations return
`IndexError::Incomplete`. No traversal, tokenization, persistence, searching,
worker scheduling, or output contract has been implemented.

## Commands

```bash
cargo test -p idiomatic-indexer-starter --test smoke --locked
cargo test -p idiomatic-indexer-solution --test smoke --locked
```

Future milestone tests keep the specification's stable filters:

```bash
cargo test -p idiomatic-indexer-solution milestone_1 --locked
cargo test -p idiomatic-indexer-solution milestone_2 --locked
```

Passing the smoke test proves that both package boundaries compile against the
same contract helper and that unfinished behavior is explicit. It does not
exercise any milestone acceptance criterion.
