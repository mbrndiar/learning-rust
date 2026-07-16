# 🔁 Comparative capstone: versioned configuration store

Rust uses the selected shared [`comparative-kv` 1.0.0
contract](spec/SPEC.md). The observable boundary is the exact `set`, `get`,
`delete`, and `list` CLI over a literal SQLite path, restricted JSON values,
global safe-integer revisions, compare-and-set expectations, migration, and
real multi-process behavior.

## Layout

```text
comparative/
├── spec/       # frozen shared specification and fixtures
├── starter/    # package: comparative-kv-starter
├── solution/   # package: comparative-kv-solution
└── tests/      # contract code compiled against both packages
```

The packages expose matching `application`, `cli`, `domain`, `error`, and
`store` modules. `KvStore` is the injectable persistence seam and
`KvApplication<S>` is the storage-independent command boundary.

- `solution/` is the complete `comparative-kv` 1.0.0 implementation. It uses
  bundled SQLite through pinned `rusqlite` 0.39.0 and passes every shared
  sequential and real-process fixture.
- `starter/` remains compileable and intentionally returns typed
  `KvError::Incomplete` values at milestone TODO boundaries. Its ignored
  milestone wrappers compile the same contracts without claiming completion.

## Commands

```bash
cargo test -p comparative-kv-solution --locked
cargo test -p comparative-kv-starter --locked
```

Run one completed solution milestone:

```bash
cargo test -p comparative-kv-solution milestone_1 --locked
```

To use the same group as a red test while implementing the starter:

```bash
cargo test -p comparative-kv-starter milestone_1 --locked -- --ignored
```

Milestones cover domain/value contracts, the exact CLI boundary, SQLite
initialization and migration, complete revision/CAS behavior, and real
multi-process contention. The process tests use repository-local temporary
directories, independent CLI children, an execution barrier, an independent
SQLite lock helper, integrity checks, and explicit database-sidecar cleanup.

Verify the frozen shared files independently:

```bash
(cd capstones/comparative/spec && sha256sum -c MANIFEST.sha256)
```
