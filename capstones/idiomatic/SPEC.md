# Idiomatic capstone specification: concurrent file indexer and search CLI

## Status and interpretation

This is the learner contract for the required Rust idiomatic capstone, equal in
weight to the comparative SQLite key/value capstone. Commands, file/index
semantics, typed failure categories, concurrency behavior, and acceptance
criteria are normative. Internal structs, module dependency direction, channel
implementation, and concrete trait implementations remain learner choices.

The removed predecessor application is not part of this normative contract. Its
reusable design lessons and historical source location are recorded in
[`../MIGRATION.md`](../MIGRATION.md).

## Bounded problem

Build a local CLI that indexes explicitly named directory roots and searches
the resulting deterministic, versioned index. The indexer:

- walks regular UTF-8 text files without following symlinks;
- tokenizes text with fixed standard-library Unicode rules;
- uses bounded worker threads with explicit ownership and cancellation;
- records recoverable per-path issues instead of abandoning the batch;
- writes a complete replacement index without exposing a partial file; and
- provides exact-term/path queries and summary statistics.

The project does not watch the filesystem, perform fuzzy/full-text ranking, use
a database, or expose mutable arbitrary keys. The index is a replaceable
domain artifact built by one process.

## Learning goals and course mapping

| Course material | Capstone outcome |
| --- | --- |
| [Modules 1–2](../../lessons/README.md) | Use expressions, control flow, loops, and `match` to implement validation and orchestration. |
| [Module 3: ownership and borrowing](../../lessons/03_ownership_and_borrowing/README.md) | Make ownership of `PathBuf`, buffers, jobs, channel messages, handles, and borrowed query data explicit. |
| [Module 4: structs, enums, patterns](../../lessons/04_structs_enums_and_patterns/README.md) | Use newtypes and exhaustive enums for roots, terms, issues, commands, and outcomes. |
| [Module 5: collections, iterators, closures](../../lessons/05_collections_iterators_and_closures/README.md) | Drive traversal/tokenization/querying with iterators, maps/sets, closures, and stable sorting. |
| [Module 6: errors, modules, I/O](../../lessons/06_errors_modules_and_io/README.md) | Own paths/files, preserve error sources, separate recoverable issues from fatal errors, and publish complete output. |
| [Module 7: generics, traits, lifetimes](../../lessons/07_generics_traits_and_lifetimes/README.md) | Define filesystem/index capabilities and use generic or trait-object boundaries deliberately. |
| [Module 8: testing](../../lessons/08_testing/README.md) | Build unit, integration, contract, temporary-tree, failure, and documentation tests. |
| [Module 9: tooling and debugging](../../lessons/09_tooling_and_debugging/README.md) | Provide a Clap CLI and pass rustfmt, Clippy-as-errors, rustdoc, MSRV, and coverage checks. |
| [Module 10: SQL and SQLite](../../lessons/10_sql_and_sqlite/README.md) | Compare an embedded relational store with the capstone's atomic versioned JSON design. |
| [Module 11: concurrency](../../lessons/11_concurrency/README.md) | Own bounded threads/channels, minimize shared state, cancel, and join cleanly. |
| [Module 12: async Rust](../../lessons/12_async_rust/README.md) | Compare the chosen thread design with async orchestration; Tokio is not required by the primary solution. |
| [Module 13: REST APIs and HTTP Clients](../../lessons/13_rest_apis_and_http_clients/README.md) | Use Serde for a validated versioned index and deterministic JSON CLI output while keeping wire types at boundaries. |

## Root and traversal contract

Roots are supplied as `NAME=PATH`. `NAME` matches
`[A-Za-z0-9][A-Za-z0-9._-]{0,31}` and is unique. `PATH` must exist, be a
readable directory at preflight, and is canonicalized only for duplicate-root
and containment checks. Canonical host paths are never written to JSON output.

Traversal rules:

- roots are processed in command order;
- job submission/completion order is not observable; final output uses the
  portable key `(root name, relative UTF-8 path)`;
- relative output paths use `/`, never start with `/`, and contain no `.` or
  `..` segments;
- symlinks are not followed and produce one `symlink_skipped` issue;
- regular files are included when their extension matches the configured set;
- default extensions are `.log`, `.md`, and `.txt`;
- `--extension .EXT` is repeatable; when supplied it replaces the defaults.
  Extensions must start with `.`, contain 1–16 ASCII alphanumeric characters
  after the dot, and compare ASCII-case-insensitively;
- hidden files/directories are included if their extension matches;
- default `--max-bytes` is `1_048_576`, valid range `1..16_777_216`;
- an oversized file, non-UTF-8 file, unreadable entry/file, disappearing file,
  non-UTF-8 relative path, or symlink is a recoverable `IndexIssue`;
- a root that fails preflight, cancellation, worker panic/protocol failure, or
  inability to publish the index is fatal.

File contents are read as strict UTF-8. A file changing during the read may
produce the bytes actually read or a recoverable read issue; snapshots and file
locking are not promised.

## Tokenization

A token is a maximal non-empty run of Rust `char::is_alphanumeric()`
characters. Normalization applies `char::to_lowercase()` to each character and
concatenates the result. Punctuation, symbols, whitespace, underscores, and
hyphens are separators.

Normalized terms of 1–64 Unicode scalar values are indexed. Longer terms are
ignored and add at most one `token_too_long` issue per document. Empty documents
are valid and have no terms. Search input is normalized by the same algorithm
and must produce exactly one term per `--term`; otherwise it is
`invalid_search_term`.

Counts are per document. Search is exact normalized-term matching; there is no
stemming, substring, phrase, regex, or relevance score.

## Version-1 index shape

JSON is compared semantically; object-member order/whitespace are not
contractual. Arrays use the stated order.

```json
{
  "schema_version": 1,
  "settings": {
    "extensions": [".log", ".md", ".txt"],
    "max_bytes": 1048576
  },
  "roots": ["fixture"],
  "documents": [
    {
      "id": 1,
      "root": "fixture",
      "path": "docs/readme.md",
      "bytes": 42,
      "terms": [
        {"term": "rust", "count": 2},
        {"term": "safe", "count": 1}
      ]
    }
  ],
  "issues": [
    {
      "root": "fixture",
      "path": "docs/link.md",
      "code": "symlink_skipped",
      "message": "symbolic links are not indexed"
    }
  ]
}
```

Invariants:

- roots remain in command order;
- documents sort by `(root order, path)` and IDs are contiguous from `1` after
  sorting, so worker completion cannot affect IDs;
- each document term array sorts by normalized term with no duplicate terms;
- issue `path` is a portable relative string, except `non_utf8_path`, for which
  it is `null`; issues sort by `(root order, path with null first, code, message)`;
- `bytes` is the number of file bytes read;
- required issue codes are `entry_unreadable`, `file_unreadable`,
  `file_disappeared`, `file_too_large`, `non_utf8_content`, `non_utf8_path`,
  `symlink_skipped`, and `token_too_long`;
- reopening validates every invariant, not just JSON syntax;
- unsupported versions, invalid IDs/order/counts/paths, duplicate documents or
  terms, and invalid normalized terms are `index_corrupt`;
- the writer creates a complete candidate in the destination directory and
  replaces the destination only after serialization/flush succeeds. A failed
  build leaves an existing valid index unchanged and cleans candidate files.

The exact temporary filename and in-memory collection strategy are not public.

## Public Rust boundary

Starter and solution crates expose equivalent documented public values for
`RootSpec`, `SearchTerm`, `DocumentId`, `IndexedDocument`, `IndexIssue`,
`IndexData`, `SearchQuery`, `SearchResult`, and a source-preserving error enum.

Behavioral capabilities:

```rust
pub trait FileTree {
    fn entries(&self, root: &RootSpec) -> Result<Box<dyn Iterator<Item = TreeEntry> + '_>, IndexError>;
    fn read(&self, entry: &TreeEntry, max_bytes: u64) -> Result<Vec<u8>, FileIssue>;
}

pub trait IndexStore {
    fn load(&self) -> Result<IndexData, IndexError>;
    fn replace(&self, index: &IndexData) -> Result<(), IndexError>;
}
```

Equivalent signatures that preserve the same injectable behavior and lifetime
constraints are acceptable if documented before implementation. The index
builder accepts an injected positive worker count and a cloneable cancellation
token with `cancel()`/`is_cancelled()` behavior. No `unsafe` code is permitted.

## Observable CLI

Run from the repository root:

```bash
cargo run -p idiomatic-indexer-solution --locked -- \
  index --index PATH --root fixture=PATH \
  [--root NAME=PATH] [--workers N] [--max-bytes N] [--extension .EXT]

cargo run -p idiomatic-indexer-solution --locked -- \
  search --index PATH --term TERM [--term TERM] \
  [--path-prefix PREFIX] [--limit N] [--format json|text]

cargo run -p idiomatic-indexer-solution --locked -- \
  stats --index PATH [--format json|text]
```

The starter package name is `idiomatic-indexer-starter`. `--workers` is
`1..64`, defaulting to available parallelism capped at `8`. `--limit` is
`1..10_000`, default `100`. Multiple terms use logical AND. `path-prefix` is a
portable relative path prefix using `/`, with no absolute/`.`/`..` segments.
All subcommands accept global `--json-errors` before the subcommand.

### `index` output

One JSON document on stdout:

```json
{
  "index": "index.json",
  "documents": 3,
  "issues": 1,
  "unique_terms": 27
}
```

The displayed index path is exactly the CLI argument, not a canonical absolute
path. Recoverable issues do not make the command fail.

### `search --format json`

```json
{
  "query": {
    "terms": ["rust", "safe"],
    "path_prefix": null,
    "limit": 100
  },
  "matches": [
    {
      "document": {
        "id": 1,
        "root": "fixture",
        "path": "docs/readme.md",
        "bytes": 42
      },
      "term_counts": [
        {"term": "rust", "count": 2},
        {"term": "safe", "count": 1}
      ]
    }
  ]
}
```

Query terms are normalized, deduplicated, and sorted. Matches sort by root
order/path and are limited afterward. Empty results succeed. Text output
contains the same fields in a golden-tested line format.

### `stats --format json`

```json
{
  "schema_version": 1,
  "roots": 1,
  "documents": 3,
  "issues": 1,
  "unique_terms": 27,
  "indexed_bytes": 512
}
```

Diagnostics go to stderr. Normal JSON stdout contains no logging.

## Failure behavior and exits

The typed error enum preserves underlying `std::io::Error` and Serde sources.
Required stable error codes:

`invalid_argument`, `invalid_root`, `duplicate_root`, `invalid_extension`,
`invalid_search_term`, `invalid_path_prefix`, `index_not_found`,
`index_corrupt`, `unsupported_index_version`, `index_read_failed`,
`index_write_failed`, `worker_failed`, and `cancelled`.

| Exit | Meaning |
| --- | --- |
| `0` | completed, including a build with recoverable issues |
| `2` | Clap usage or validation error |
| `3` | root/traversal preflight error |
| `4` | index read/corruption/version error |
| `5` | index write or worker protocol/panic error |
| `130` | cancelled build |

With `--json-errors`, stderr contains one semantic object:

```json
{"error":{"code":"index_corrupt","message":"...","details":{}}}
```

Without it, errors remain concise and source chained. Panics are bugs, not
ordinary error handling. On the first fatal worker error, stop scheduling,
signal cancellation, drain/close channels as required, join every started
worker, and return the fatal error.

## Five guided milestones

### Milestone 1 — validated domain

Implement newtypes/enums, tokenization, in-memory indexing/querying, ordering,
and typed errors.

Acceptance:

- invalid roots/terms/paths cannot enter validated APIs;
- tokenization covers Unicode case expansion, separators, limits, and empty text;
- in-memory query uses AND semantics and stable ordering;
- exhaustive matches compile without wildcard catch-alls for closed domain enums;
- milestone 1 unit/contracts pass.

### Milestone 2 — traversal and CLI

Implement injected traversal/read capabilities, deterministic entry handling,
recoverable issues, Clap commands, and text/JSON boundaries.

Acceptance:

- an in-memory tree covers symlinks, unreadable entries, non-UTF-8 paths, and
  disappearance without platform permissions;
- real temporary trees cover ordinary traversal;
- root/path containment and extension rules are enforced;
- stdout/stderr/exits match golden cases;
- milestone 2 integration tests pass.

### Milestone 3 — versioned index

Implement Serde envelope validation, invariant checking, load/search/stats, and
publish-after-complete replacement.

Acceptance:

- create/reopen round trips preserve semantic data;
- every listed corruption invariant has a fixture/test;
- injected serialize/write/replace failure preserves the previous index and
  cleans candidates;
- unsupported versions fail closed;
- milestone 3 storage contracts pass.

### Milestone 4 — bounded concurrency

Implement worker ownership, bounded job/result flow, cancellation, deterministic
collection, and complete joins.

Acceptance:

- active reads never exceed `workers`;
- a barrier forces reverse completion while index bytes remain semantically
  identical to the single-worker result;
- cancellation and fatal worker failure stop new scheduling and join all workers;
- shared mutable state is absent or narrowly synchronized;
- repeated concurrency tests finish without hangs or leaked threads.

### Milestone 5 — full integration

Complete temporary-tree/subprocess tests, optional platform cases, docs, MSRV,
Clippy, coverage, and reproducibility.

Acceptance:

- CLI index/search/stats fixtures pass as child processes;
- Linux portable cases include symlink handling; non-UTF-8 path/permission cases
  may use the fake on unsupported platforms;
- `cargo check` and Clippy pass on Rust 1.85;
- fmt, unit/integration/doc tests and coverage report pass with `--locked`;
- no test scans outside its explicit fixture roots.

## Starter, solution, and test architecture

```text
capstones/idiomatic/
├── SPEC.md
├── starter/                 # crate: idiomatic-indexer-starter
│   └── src/{lib.rs,main.rs,...}
├── solution/                # crate: idiomatic-indexer-solution
│   └── src/{lib.rs,main.rs,...}
└── tests/
    ├── contracts/
    └── fixtures/
```

Both crates expose matching public modules for domain, tokenization, tree
access, index building/storage, query, and CLI execution. Shared generic
contract functions are compiled into both integration suites. Starter types,
docs, signatures, and Clap surface are complete; unfinished bodies use scoped
`todo!()` so `cargo check --workspace --all-targets` remains meaningful.
Behavioral solution tests do not require the starter to pass unfinished cases.

## Deterministic fixtures and seams

Required fixtures:

- a small tree manifest with nested `.txt`, `.md`, ignored extension, empty,
  Unicode, oversized, and symlink entries;
- expected semantic index/search/stats JSON;
- corrupt indexes for syntax, header/version, ID gap, duplicate path/term,
  invalid normalized term, and unsafe relative path;
- fake-tree entries for non-portable permission and non-UTF-8 path cases.

Required seams are `FileTree`, `IndexStore`, worker count/spawner or equivalent,
and cancellation token. Tests use `tempfile`, barriers/channels, fixed content,
sorted semantic comparisons, and repository fixtures. No home-directory scan,
network, randomness, wall clock, sleep-based race assertion, or benchmark gate
is allowed.

## Dependencies and supported runtime

Supported language/toolchain:

- Rust edition `2024`;
- minimum supported Rust `1.85`;
- stable Rust for the main quality job.

Permitted workspace manifest requirements:

- Clap `4.5` with `derive`;
- Serde `1.0` with `derive`;
- `serde_json` `1.0`;
- `tempfile` `3.20`;
- `thiserror` `2.0`.

`Cargo.lock` is the authority for exact resolved versions, and repository gates
use `--locked`. The capstone adds no package-specific dependency. The workspace's
Tokio `1.46` requirement remains available to course material but is rejected
for the primary capstone implementation; use `std::thread` and `std::sync`.
Also rejected:
`walkdir`, Rayon, regex crates, signal crates, full-text engines, databases,
memory mapping, Unicode segmentation/stemming libraries, and checksum crates.

The required behavior is portable across Linux, macOS, and Windows. Symlink,
permission, and non-UTF-8 path tests must be capability-gated and backed by the
portable fake contracts rather than weakening semantics.

## Exclusions

No filesystem watching, incremental merge, content snippets, regex/phrase/fuzzy
query, ranking, stemming, binary parsing, archive traversal, symlink following,
mmap, hashing/deduplication by content, database, distributed index, concurrent
writers, production performance target, or async primary implementation.

## Quality and coverage commands

Focused:

```bash
cargo test -p idiomatic-indexer-solution milestone_1 --locked
cargo test -p idiomatic-indexer-solution --locked
```

Final validation:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --lib --bins --locked
cargo test -p idiomatic-indexer-solution --locked
cargo test --doc --workspace --locked
cargo llvm-cov -p idiomatic-indexer-solution \
  --all-targets --summary-only --locked
```

The repository reports rather than enforces a numeric Rust coverage threshold.
Every public failure category and milestone acceptance path requires a test.

## Relationship to the historical predecessor

Reuse these architectural habits from the predecessor project:

- validated private-field newtypes and custom deserialization;
- source-preserving `thiserror` categories;
- trait/generic capability injection;
- candidate-state atomic replacement and temporary-directory tests;
- Clap/library/main separation, integration/doc tests, workspace/lint wiring,
  and the documented single-writer limitation.

Replace vector CRUD and Task persistence with tree traversal, maps/sets,
tokenization, index invariants, and worker protocols. The durable side-by-side
mapping and historical source location are recorded in
[`../MIGRATION.md`](../MIGRATION.md).
