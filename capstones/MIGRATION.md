# 🧭 From the historical Task Manager to the capstone track

Task Manager's last pre-removal commit was
[`7f4f776c771e965aa3b9b53b332019c83e9bb2eb`](https://github.com/mbrndiar/learning-rust/tree/7f4f776c771e965aa3b9b53b332019c83e9bb2eb/project/task_manager).
Its last in-repository path was `project/task_manager`. Use that snapshot only
when historical source context is useful; the comparative and idiomatic
capstones are the maintained assessment tracks.

## Choose the next project

- Use **comparative-kv** to practice an exact cross-language contract, SQLite
  migration, revisions, compare-and-set behavior, and real process contention.
- Use the **idiomatic indexer** to practice Rust-specific ownership, bounded
  threads, cancellation, deterministic collections, and filesystem seams.

## Durable concept map

| Task Manager concept | Comparative key/value capstone | Idiomatic indexer capstone |
| --- | --- | --- |
| `TaskId` and private `Task` fields protect invariants | `Key`, `Revision`, and expectation enums validate the shared contract | `RootSpec`, `SearchTerm`, `DocumentId`, settings, and issue enums validate inputs and persisted data |
| `TaskStore` isolates persistence | `KvStore` isolates SQLite from `KvApplication<S>` | `FileTree` isolates traversal and `IndexStore` isolates publication |
| `TaskManager<S>` owns storage-independent operations | `KvApplication<S>` owns command dispatch over an injected store | `IndexBuilder<F, C>` owns bounded orchestration over injected tree and cancellation capabilities |
| A versioned Serde envelope is validated after decoding | SQLite schema metadata is validated and legacy v0 is migrated transactionally | Versioned JSON is decoded, every index invariant is revalidated, then queried |
| Candidate state is written and atomically replaces the JSON file | SQLite transactions, revisions, expectations, and busy handling define multi-process mutations | A complete deterministic index candidate replaces the old index only after serialization and flush succeed |
| One behavior contract runs against memory and file stores | Shared fixtures and contract functions exercise starter and solution packages, including child processes | Shared contracts exercise starter and solution packages with fake trees, real trees, storage failures, and concurrency |
| The CLI delegates to a library and keeps output at the boundary | The exact JSON stdout/stderr/exit contract is language-neutral | Text/JSON reports and typed exits wrap Rust-specific indexing internals |
| Single-writer persistence is an explicit limitation | Concurrent writers are part of the revision/CAS contract | Index publication remains single-writer while file reads use bounded worker threads |

## Migration method

1. Preserve the architectural habits: validated domain values, capability
   traits, source-preserving errors, deterministic tests, and a thin `main`.
2. Do not copy the predecessor's `Task`, vector CRUD, or JSON storage schema into
   the selected capstone. Start from its `SPEC.md` and public types.
3. Implement one starter milestone, run its ignored contract as a red test, and
   remove only that milestone wrapper's `#[ignore]` after it passes. Comparative
   subprocess helper entry points remain ignored because the contracts invoke
   them explicitly.
4. Compare the solution only after the matching contract is green, then run the
   package and repository gates from [`capstones/README.md`](README.md).

The historical source is not a workspace member and is no longer built, tested,
documented, or covered in CI.
