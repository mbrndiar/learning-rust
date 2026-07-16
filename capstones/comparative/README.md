# 🔁 Comparative capstone: versioned configuration store

Rust uses the selected shared [`comparative-kv` 1.0.0
contract](spec/SPEC.md). The observable boundary is the exact `set`, `get`,
`delete`, and `list` CLI over a literal SQLite path, restricted JSON values,
global safe-integer revisions, compare-and-set expectations, migration, and
real multi-process behavior.

## Scaffold layout

```text
comparative/
├── spec/       # frozen shared specification and fixtures
├── starter/    # package: comparative-kv-starter
├── solution/   # package: comparative-kv-solution
└── tests/      # contract code compiled against both packages
```

The packages expose matching `application`, `cli`, `domain`, `error`, and
`store` modules. `KvStore` is the injectable persistence seam and
`KvApplication<S>` is the storage-independent command boundary. Public
operations currently return `KvError::Incomplete`; this is intentional and
keeps incomplete behavior typed rather than pretending to conform.

No SQLite provider or milestone behavior is present. In particular,
`rusqlite` is deliberately deferred to the implementation pilot.

## Commands

```bash
cargo test -p comparative-kv-starter --test smoke --locked
cargo test -p comparative-kv-solution --test smoke --locked
```

Name future tests by milestone so these filters stay stable:

```bash
cargo test -p comparative-kv-solution milestone_1 --locked
cargo test -p comparative-kv-solution milestone_2 --locked
```

Passing the smoke test proves only that the public scaffold and typed incomplete
boundary are available. It does not claim conformance with the shared fixtures.
