# 🔁 Comparative capstone: versioned configuration store

Rust uses the selected shared [`comparative-kv` 1.0.0
contract](spec/SPEC.md). The observable boundary is the exact `set`, `get`,
`delete`, and `list` CLI over a literal SQLite path, restricted JSON values,
global safe-integer revisions, compare-and-set expectations, migration, and
real multi-process behavior.

Complete [Module 10: SQL and SQLite](../../lessons/10_sql_and_sqlite/README.md)
before starting; the capstone extends its schema, transaction, locking, and
repository ideas into a frozen multi-process contract.

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
- `starter/` is compilable and intentionally returns typed storage-category
  errors naming unfinished capabilities at milestone boundaries. Its ignored
  milestone wrappers compile the same contracts without claiming completion.

The `rusqlite` `bundled` feature builds SQLite from source. A system SQLite
library is not required, but a working C compiler, linker, and platform build
tools are required even though the application code is Rust. All packages
inherit edition 2024 and the Rust 1.85 MSRV from the workspace.

## Five milestones

1. **Domain and value contracts** — validate keys, expectations, safe revisions,
   restricted JSON, normalization, and structured errors without touching
   storage for rejected input.
2. **Application and CLI boundary** — implement the exact command grammar,
   validation precedence, one-line JSON envelopes, stderr discipline, and exit
   mapping against an injected store.
3. **SQLite initialization and migration** — open literal paths, configure the
   connection, create and validate the v1 schema, migrate the one supported
   legacy shape transactionally, and preserve invalid storage unchanged.
4. **Revisions and complete mutations** — implement compare-and-set
   expectations, global revisions, atomic set/delete transactions, conflict and
   not-found precedence, exhaustion, and deterministic listing.
5. **Real-process integration** — verify initialization and migration races,
   contention, competing mutations, busy timeouts, process cleanup, and SQLite
   sidecar cleanup with independent child processes.

Each solution milestone is an active contract group. Run the matching ignored
starter group as a red feedback loop, implement only that scope, then remove its
milestone wrapper's `#[ignore]` after it passes. The normative acceptance details
remain in [`spec/SPEC.md`](spec/SPEC.md#11-learner-milestones-and-acceptance-criteria).

## Commands

```bash
cargo test -p comparative-kv-solution --locked
cargo test -p comparative-kv-starter --locked
```

Run one completed solution milestone:

```bash
cargo test -p comparative-kv-solution milestone_1 --locked
```

Use the same group as a red test while implementing the starter:

```bash
cargo test -p comparative-kv-starter milestone_1 --locked -- --ignored
```

The command is expected to fail until milestone 1 is implemented. Once it is
green, remove that milestone wrapper's `#[ignore]`; leave the ignored subprocess
helper entry points alone.

Milestones cover domain/value contracts, the exact CLI boundary, SQLite
initialization and migration, complete revision/CAS behavior, and real
multi-process contention. The process tests use repository-local temporary
directories, independent CLI children, an execution barrier, an independent
SQLite lock helper, integrity checks, and explicit database-sidecar cleanup.

Verify the frozen shared files independently:

```bash
(cd capstones/comparative/spec && sha256sum -c MANIFEST.sha256)
```
