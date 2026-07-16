# Comparative capstone: versioned configuration store

**Specification identifier:** `comparative-kv`  
**Specification version:** `1.0.0`  
**Status:** frozen observable contract

## 1. Scope and conformance

This specification defines the language-neutral, observable contract for a
local SQLite-backed versioned key-value configuration store. A conforming
implementation provides exactly four commands: `set`, `get`, `delete`, and
`list`.

The contract standardizes command-line behavior, accepted data, JSON response
envelopes, persistence, revisions, migration, and multi-process behavior. It
does not standardize internal modules, types, abstractions, dependency
injection, SQLite bindings, or test frameworks.

Version 1 deliberately excludes networking, HTTP, prefix queries, namespaces,
batch operations, transactions spanning commands, TTLs, watches, import/export,
encryption, access control, remote synchronization, and secret-management or
production-readiness claims.

The files under `fixtures/` and the rules in `SCENARIOS.md` are normative. When
prose and a fixture disagree, the prose is authoritative and the fixture is a
specification defect that requires a new shared-spec release.

Normative terms **MUST**, **MUST NOT**, **SHOULD**, and **MAY** have their usual
requirements-language meanings.

## 2. Process and CLI contract

`PROGRAM` below means the language-local executable launcher. The launcher
itself is not shared. Everything after it is shared and case-sensitive.

```text
PROGRAM --db PATH set KEY --value-json JSON [--expect EXPECTATION]
PROGRAM --db PATH get KEY
PROGRAM --db PATH delete KEY [--expect EXPECTATION]
PROGRAM --db PATH list
```

These are the only accepted forms.

- `--db PATH` is required exactly once and MUST precede the command.
- `--value-json JSON` is required exactly once for `set`.
- `--expect` is optional and MUST occur after the command's required
  arguments.
- Options use separate arguments. `--db=PATH`, `--expect=VALUE`, aliases,
  combined short options, and environment-variable substitutes are invalid.
- An omitted set expectation is `any`.
- An omitted delete expectation is `any`.
- Set accepts `any`, `absent`, or an exact revision.
- Delete accepts `any` or an exact revision. `absent` is invalid for delete.
- An exact revision is canonical decimal text matching
  `[1-9][0-9]*` and no greater than `9007199254740991`. Signs, whitespace,
  leading zeroes, fractions, and exponent notation are invalid expectations.
- Additional positional arguments, duplicated options, unknown options,
  unknown commands, and missing arguments are usage errors.
- Command and option names are ASCII and case-sensitive.

The process MUST produce one response and terminate. It MUST NOT prompt, read
stdin, start a server, or enter an interactive mode.

### 2.1 Database path

`PATH` is one nonempty command-line argument.

- It is treated as a literal operating-system filesystem path.
- Relative paths are resolved by the operating system against the process's
  current working directory.
- The implementation MUST NOT expand `~`, environment variables, glob
  characters, or URI escapes.
- SQLite URI mode MUST be disabled.
- The exact values `:memory:` and any path beginning with the case-sensitive
  prefix `file:` are unsupported forms and MUST be rejected before opening
  storage.
- The parent directory MUST already exist. The command does not create parent
  directories.
- The supported target is an absent file or a regular SQLite database file on
  a writable, same-host local filesystem.
- Paths to directories, special files, network filesystems, synchronized
  folders with nonstandard locking, and symlink-dependent layouts are outside
  the conformance environment.
- The conformance fixtures use scalar Unicode paths that the host runtime can
  represent and include paths with spaces and leading dashes in individual
  components. A leading dash in `PATH` is data because it follows `--db`.

An absent database file is initialized as specified in section 8.

### 2.2 Validation precedence

Errors are selected in this order:

1. Parse the CLI grammar and validate the database-path form.
2. Validate the key and expectation, when present.
3. For `set`, check the UTF-8 byte limit, parse JSON, and validate the complete
   value tree.
4. Open, configure, inspect, initialize, or migrate storage.
5. Execute the command.

Therefore invalid command input returns an exit-2 response even if the database
parent is missing or the database is malformed. Storage is not created or
changed when steps 1-3 fail.

## 3. Keys

Keys are case-sensitive opaque strings matching this entire ASCII regular
expression:

```text
[A-Za-z0-9][A-Za-z0-9._/-]{0,127}
```

Consequences:

- Length is 1 through 128 ASCII bytes and characters.
- The first character is alphanumeric.
- Later characters may additionally contain `.`, `_`, `/`, and `-`.
- `/` has no hierarchy or filesystem meaning.
- No trimming, case folding, path normalization, or Unicode normalization is
  performed.
- Any non-ASCII character is invalid.

## 4. Restricted JSON value model

`set --value-json` accepts one RFC 8259 JSON text, subject to the restrictions
below.

### 4.1 Input size and depth

- The argument's UTF-8 encoding MUST be at most 65,536 bytes, including JSON
  syntax and insignificant whitespace.
- A scalar root has container depth 0.
- Entering an array or object increases container depth by 1.
- The maximum container depth is 32. A container at depth 33 is invalid.

### 4.2 Supported values

The value tree may contain:

- `null`;
- booleans;
- strings containing Unicode scalar values;
- arrays;
- objects whose member names contain Unicode scalar values; and
- numbers satisfying section 4.3.

Strings and object names MUST NOT contain unpaired UTF-16 surrogate code
points, whether introduced by `\u` escapes or by a runtime's string
representation. A valid escaped surrogate pair denotes its single
supplementary Unicode scalar value.

No Unicode normalization occurs. Canonically equivalent scalar sequences
remain distinct. Object member names are case-sensitive.

If an object contains duplicate member names, the last member in source order
wins. The normalized value contains one member with that name. Duplicate
detection uses exact Unicode scalar sequences without normalization.

Object member order is not observable. Tests compare objects semantically.
Array order is observable.

### 4.3 Numbers

JSON number tokens are validated using both their exact decimal meaning and
their IEEE-754 binary64 interpretation:

1. The token MUST convert to a finite binary64 value. A token such as `1e400`
   that overflows binary64 is invalid.
2. The exact mathematical decimal value denoted by the token MUST be an
   integer.
3. That integer MUST be in
   `[-9007199254740991, 9007199254740991]`.

Thus general decimals are not supported. `1.0` and `1e2` are accepted because
their exact values are integral. `1.5`, `1e-1`,
`9007199254740991.1`, `9007199254740992`, `NaN`, and `Infinity` are
rejected. The last two are also outside the JSON grammar.

Accepted numbers are stored and emitted as canonical base-10 integers with no
leading plus sign, leading zeroes, decimal point, or exponent. Every spelling
of negative zero normalizes to `0`.

### 4.4 Normalized JSON

Before persistence, the implementation MUST construct the normalized semantic
value described above. `value_json` stores a compact valid JSON encoding of
that value.

- Numbers use canonical integer spelling.
- Duplicate object members have been collapsed by last-member-wins.
- Strings preserve Unicode scalar sequences.
- Array order is preserved.
- Object member order and escape spelling are implementation-defined.
- No insignificant whitespace is stored.

Byte-for-byte stored JSON is not compared across languages. A later `get` or
`list` MUST return the same normalized semantic value.

## 5. Output envelope and process behavior

Every invocation, including every failure covered by this specification, MUST:

- write exactly one UTF-8 JSON object followed by one line-feed byte (`\n`) to
  stdout;
- write no byte-order mark, prompt, warning, progress record, or other stdout
  content;
- write nothing to stderr in conformance mode; and
- exit with the code assigned in section 6.

The JSON object MUST be compact: no insignificant whitespace outside strings.
Object member order and JSON string escape spelling are not normative. Parsed
objects are compared semantically. Array order, normalized integer spelling,
the single-line requirement, and the final line feed are normative.

Success has exactly these top-level members:

```json
{"ok":true,"result":null}
```

Failure has exactly these top-level members:

```json
{"ok":false,"error":{"category":"category","details":{}}}
```

No additional top-level or error members are allowed. In the command shapes
below, `VALUE` means the normalized JSON value and `REVISION` is a JSON integer.

### 5.1 `set`

```json
{"ok":true,"result":{"key":"KEY","value":"VALUE","revision":1,"created":true}}
```

`value` is not necessarily a string; the example is schematic. `created` is
true when the key was absent immediately before this successful mutation and
false when it was overwritten. A successful set always consumes a revision,
even when the normalized new value is semantically equal to the old value.

### 5.2 `get`

```json
{"ok":true,"result":{"key":"KEY","value":"VALUE","revision":1}}
```

### 5.3 `delete`

```json
{"ok":true,"result":{"key":"KEY","deleted_revision":1,"revision":2}}
```

`deleted_revision` is the entry's last-set revision. `revision` is the new
global deletion commit revision. No tombstone is retained.

### 5.4 `list`

```json
{"ok":true,"result":{"entries":[],"global_revision":0}}
```

Each entry has exactly the `get` result shape:

```json
{"key":"KEY","value":"VALUE","revision":1}
```

Entries are sorted by ascending SQLite `BINARY` key order. Because keys are
ASCII, this is unsigned ASCII byte lexicographic order: a strict prefix sorts
first, digits precede uppercase letters, and uppercase letters precede
lowercase letters. Object member order inside values remains non-normative.

## 6. Errors and exit codes

The following categories, detail objects, and exit codes are exhaustive for
the shared contract. Prose messages are not part of the envelope.

### 6.1 Exit 2: command or value validation

| Category | Exact `details` shape | Meaning |
| --- | --- | --- |
| `usage` | `{"reason":"invalid_cli"}` | The invocation does not match section 2. |
| `invalid_argument` | `{"field":"db","reason":"empty"}` | `PATH` is empty. |
| `invalid_argument` | `{"field":"db","reason":"unsupported_form"}` | `:memory:` or `file:` URI form. |
| `invalid_argument` | `{"field":"key","reason":"format"}` | The key violates section 3. |
| `invalid_argument` | `{"field":"expect","reason":"format"}` | The expectation is not allowed or its revision spelling/range is invalid. |
| `invalid_json` | `{"reason":"syntax"}` | The argument is not one complete RFC 8259 JSON text. |
| `invalid_value` | `{"reason":"byte_limit"}` | UTF-8 input is larger than 65,536 bytes. |
| `invalid_value` | `{"reason":"depth_limit"}` | Container depth is greater than 32. |
| `invalid_value` | `{"reason":"unpaired_surrogate"}` | A string or object name contains an unpaired surrogate. |
| `invalid_value` | `{"reason":"non_integral_number"}` | A number's exact decimal value is not integral. |
| `invalid_value` | `{"reason":"non_finite_number"}` | A valid JSON number token overflows binary64. |
| `invalid_value` | `{"reason":"number_out_of_range"}` | An integral number is outside the safe-integer range. |

For a value containing multiple defects, deterministic tree traversal selects
the first defect: arrays in index order and objects in last-member-wins source
order. At a number, checks occur in the order listed in section 4.3. The byte
limit precedes parsing; JSON syntax precedes tree validation; depth is checked
when entering each container; a string/name surrogate check precedes visiting
that member's value.

### 6.2 Exit 3: expectation conflict

Category `conflict` has:

```json
{"key":"KEY","expected":"absent","actual":7}
```

or:

```json
{"key":"KEY","expected":5,"actual":7}
```

`expected` is exactly `"absent"` or the requested revision. `actual` is the
current entry revision or `null` when an exact-revision set expected an entry
that is absent.

### 6.3 Exit 4: missing key

Category `not_found` has:

```json
{"key":"KEY"}
```

It applies to `get` of an absent key and to every `delete` of an absent key,
including delete with an exact expectation. A missing delete is checked before
delete expectation matching.

### 6.4 Exit 5: storage, schema, busy, or revision failure

| Category | Exact `details` shape | Meaning |
| --- | --- | --- |
| `busy` | `{"timeout_ms":10000}` | SQLite could not acquire the required lock after its configured wait. |
| `unsupported_schema` | `{"found":N,"supported":1}` | Recognizable metadata declares a version greater than 1. |
| `invalid_storage` | `{"reason":"malformed_schema"}` | Tables, columns, constraints, metadata rows, or application objects do not match v0 or v1. |
| `invalid_storage` | `{"reason":"invalid_key","key":"KEY"}` | A stored v0/v1 key violates section 3. |
| `invalid_storage` | `{"reason":"invalid_value","key":"KEY"}` | Stored JSON is invalid or outside section 4. |
| `invalid_storage` | `{"reason":"revision_invariant"}` | v1 revision metadata or entry revisions violate section 8.3. |
| `invalid_storage` | `{"reason":"integrity_check_failed"}` | SQLite reports database corruption. |
| `revision_exhausted` | `{"maximum":9007199254740991}` | A successful mutation would exceed the global safe-integer maximum. |
| `storage_error` | `{"operation":"OPERATION","reason":"storage_failure"}` | Another host I/O or SQLite failure occurred. |

`OPERATION` is exactly one of `open`, `configure`, `initialize`, `migrate`,
`read`, `write`, or `commit`, chosen for the phase that failed. Underlying
provider text, paths, stack traces, and platform error numbers are not exposed
in the normative envelope.

If recognizable metadata reports a future version, `unsupported_schema` takes
precedence over other structural checks. Otherwise corruption detected by an
integrity check takes precedence over structural `invalid_storage`.

## 7. Revision and expectation semantics

Revisions are global commit sequence numbers represented as JSON safe integers.

- An empty v1 database has `global_revision = 0`.
- Each successful `set` or `delete` increments `global_revision` exactly once.
- A successful set stores the new global revision on that entry.
- A successful delete returns the deletion revision but stores no tombstone.
- Failed validation, not-found operations, conflicts, busy timeouts, storage
  failures, revision exhaustion, and rolled-back transactions consume no
  revision.
- Deleting and recreating a key produces a later revision.
- There is no per-key counter and revisions are never reused.

Set expectations:

- `any`: always passes; creates or overwrites.
- `absent`: passes only when the key is absent.
- exact `N`: passes only when the key exists at revision `N`.
- An absent exact-revision set is `conflict` with `actual:null`.

Delete expectations:

- Missing key is always `not_found`.
- `any`: passes for any existing key.
- exact `N`: passes only when the existing key is at revision `N`.

Expectation matching occurs before revision-exhaustion checking. Therefore a
conflict or missing delete remains a conflict/not-found even when the global
revision is already at its maximum.

## 8. SQLite persistence contract

### 8.1 Connection configuration

Each process MUST:

- open the literal path with SQLite URI processing disabled;
- configure a SQLite busy timeout of at least 10,000 milliseconds before any
  operation that may contend;
- request `PRAGMA journal_mode=WAL` for the supported local database;
- enable foreign-key enforcement even though v1 has no foreign keys; and
- close all statements, transactions, and connections before process exit.

The observable timeout detail remains `10000`; bindings MAY wait slightly
longer but MUST NOT fail earlier because of a shorter configured timeout.
SQLite's normal scheduling jitter is not an observable duration guarantee.

`WAL`, `-wal`, and `-shm` behavior is supported only for same-host local files.

### 8.2 Exact v1 application schema

The v1 database has exactly these application tables and no application
triggers, views, or explicit indexes. SQLite-owned objects such as primary-key
autoindexes are allowed.

```sql
CREATE TABLE store_metadata (
    singleton       INTEGER PRIMARY KEY CHECK (singleton = 1),
    schema_version  INTEGER NOT NULL CHECK (schema_version = 1),
    global_revision INTEGER NOT NULL
                    CHECK (global_revision BETWEEN 0 AND 9007199254740991)
);

CREATE TABLE entries (
    key        TEXT PRIMARY KEY COLLATE BINARY,
    value_json TEXT NOT NULL CHECK (json_valid(value_json)),
    revision   INTEGER NOT NULL
               CHECK (revision BETWEEN 1 AND 9007199254740991)
);
```

There is exactly one metadata row:

```sql
INSERT INTO store_metadata(singleton, schema_version, global_revision)
VALUES (1, 1, 0);
```

Equivalent whitespace and identifier quoting in `sqlite_schema.sql` are
allowed. Column names, declared types, nullability, primary keys, collation,
checks, and table/object set are not optional.

`PRAGMA user_version` and `PRAGMA application_id` are not version authorities
and MUST remain at their SQLite defaults.

### 8.3 v1 invariants

On open, a conforming implementation validates:

- exactly one metadata row with `singleton = 1` and `schema_version = 1`;
- `global_revision` is in the safe range;
- every key and normalized stored value satisfies sections 3 and 4;
- every entry revision is in `1..global_revision`; and
- live entry revisions are unique.

Violation is `invalid_storage`. Opening malformed storage MUST NOT repair,
drop, or partially rewrite it.

An implementation SHOULD run an appropriate SQLite integrity check before
reporting structural validity. A failed check is
`invalid_storage/integrity_check_failed`.

### 8.4 Fresh initialization

A database is fresh only when it contains no non-SQLite application tables,
views, triggers, or indexes.

Initialization is atomic under a write-reserving transaction:

1. Acquire the equivalent of `BEGIN IMMEDIATE`.
2. Reinspect the schema after acquiring the lock.
3. If it is still fresh, create the exact v1 schema and metadata row.
4. If another process already initialized or migrated it, validate that result
   instead.
5. Commit.

Two or more processes racing to initialize one absent database MUST all
complete against one valid empty v1 database or return `busy` after the
normative wait. With the fixture's lock hold times they must all succeed.

### 8.5 Legacy v0 recognition and migration

The sole supported legacy schema contains exactly one application table and no
other application objects:

```sql
CREATE TABLE entries (
    key        TEXT PRIMARY KEY COLLATE BINARY,
    value_json TEXT NOT NULL
);
```

Migration occurs inside one `BEGIN IMMEDIATE` transaction and MUST recheck the
schema after acquiring the lock:

1. Validate every v0 key and value using sections 3 and 4.
2. Read rows in ascending `BINARY` key order.
3. Create the v1 schema without exposing a partial state.
4. Normalize every value and copy rows in that order, assigning entry
   revisions `1` through `N`.
5. Insert metadata with `global_revision = N`.
6. Remove the legacy table/name and commit.

An empty v0 database migrates with global revision 0. Any validation or storage
failure rolls back completely and leaves the original v0 schema and rows
unchanged. Migration is the only operation that can create multiple historical
revision numbers inside one transaction.

Two processes racing to migrate MUST both observe the same completed v1 result;
the second process validates v1 after acquiring its lock.

No other legacy shape is supported. A lower, absent, or malformed version that
is neither exact v0 nor exact v1 is `invalid_storage/malformed_schema`. A
recognizable future metadata version is `unsupported_schema`.

### 8.6 Mutation transactions

Every set and delete uses one independent write transaction with behavior
equivalent to SQLite `BEGIN IMMEDIATE`. Schema initialization/migration, if
needed, completes first; the command mutation then uses its own immediate
transaction.

Set transaction:

```text
BEGIN IMMEDIATE
read the current entry revision
check the set expectation
on mismatch: ROLLBACK and return conflict
if global_revision is maximum: ROLLBACK and return revision_exhausted
increment global_revision exactly once
insert or update normalized value_json and entry revision
COMMIT
return the committed result
```

Delete transaction:

```text
BEGIN IMMEDIATE
read the current entry revision
if absent: ROLLBACK and return not_found
check the delete expectation
on mismatch: ROLLBACK and return conflict
if global_revision is maximum: ROLLBACK and return revision_exhausted
increment global_revision exactly once
delete the entry
COMMIT
return deleted_revision and the committed deletion revision
```

The success envelope is emitted only after commit succeeds. A commit failure is
a storage failure and consumes no observable revision.

## 9. Multi-process and busy behavior

Conformance uses independent operating-system child processes that share one
database path. Multiple calls within one process, threads, jobs that share one
connection, or mocked storage do not satisfy process scenarios.

Because all mutations reserve the write lock before reading expectations:

- distinct writers serialize into one total global revision order;
- same-key exact-revision races have exactly one winner;
- losers observe the winner's committed revision and return conflict;
- conflicts and busy failures do not create gaps; and
- readers never observe a partially initialized, migrated, or mutated state.

If an external process holds an immediate write transaction:

- a command whose lock is released before the busy timeout waits and then
  proceeds normally; or
- a command that cannot acquire the lock after the configured wait returns
  category `busy`, exit 5, without changing storage.

The implementation MUST NOT hang indefinitely, emit malformed/multiple JSON
records, or map an ordinary expectation race to `busy`.

## 10. Supported environment and cleanup

The shared test environment assumes:

- one host and one writable local filesystem;
- ordinary SQLite advisory/file locks and WAL shared-memory behavior;
- wall-clock scheduling without a guaranteed process order;
- process timeouts supplied by the language-local runner; and
- no process crash, disk-full injection, power-loss simulation, hostile
  database modification during a command, or network filesystem.

Each scenario MUST use a fresh dedicated directory. On completion, including
failure, the runner MUST:

1. terminate only child processes it started;
2. wait for every child and lock-helper process;
3. close every SQLite connection;
4. remove the database file and any same-basename `-wal`, `-shm`, and
   `-journal` siblings; and
5. remove the scenario directory only after verifying those files are closed.

Cleanup failures fail the scenario. Tests MUST NOT delete a shared parent
directory or rely on a pre-existing user database.

## 11. Learner milestones and acceptance criteria

Implementations use five progressive milestone groups. Internal APIs remain
language-local.

### Milestone 1: domain and value contracts

Implement keys, expectations, safe revisions, restricted JSON parsing and
normalization, semantic value comparison, and typed/structured errors.

Acceptance:

- every case in `fixtures/keys.json`,
  `fixtures/values-accepted.json`, and
  `fixtures/values-rejected.json` passes;
- duplicate members, surrogate handling, exact-decimal number checks, depth,
  byte limits, and no-normalization cases are covered; and
- no filesystem or SQLite access occurs for rejected domain input.

### Milestone 2: application boundary

Implement the exact CLI grammar, validation precedence, one-object envelopes,
stderr discipline, result shapes, and exit-code mapping against a deterministic
application/store seam.

Acceptance:

- CLI cases in `fixtures/scenarios/invalid.json` pass;
- sequential command result shapes are implemented;
- stdout is one compact JSON line with a final LF and stderr is empty; and
- the launcher exposes no additional normative commands or output modes.

### Milestone 3: SQLite initialization and migration

Implement literal path handling, connection configuration, exact v1 schema,
fresh initialization, v1 validation, exact v0 migration, rollback, and ordinary
round trips.

Acceptance:

- `normal.json` and `migration.json` sequential cases pass;
- a fresh `list` creates valid empty v1 storage;
- migration ordering and assigned revisions are deterministic;
- invalid/future storage is not silently repaired; and
- reopening preserves normalized semantic values and revisions.

### Milestone 4: revisions, expectations, and complete mutations

Implement global revisions, all set/delete expectations, immediate
transactions, conflict/not-found precedence, exhaustion, ordering, and
no-revision-on-failure guarantees.

Acceptance:

- all sequential normal and boundary scenarios pass;
- same-value sets consume a revision;
- failed conflicts, missing deletes, and exhaustion leave state unchanged;
- delete/recreate behavior is correct; and
- `list` is in exact binary-key order with the current global revision.

### Milestone 5: real process integration

Implement and test independent-process initialization, migration, contention,
CAS/delete races, busy waiting/timeouts, and cleanup.

Acceptance:

- every scenario in `fixtures/scenarios/multiprocess.json` runs with real child
  processes and passes for the required repeat count;
- winner identity and scheduling order are never predicted;
- busy cases neither hang nor consume revisions;
- all child processes and WAL sidecars are cleaned up; and
- the repository's existing format, lint/static-analysis, build/type-check, and
  test gates pass for the solution.

## 12. Shared-spec drift control

All six repositories carry a byte-identical
`capstones/comparative/spec/` tree.

- `SPEC_VERSION` contains only `1.0.0` plus LF.
- `MANIFEST.sha256` contains SHA-256 hashes for every regular file in the tree
  except `MANIFEST.sha256` itself.
- Manifest paths are slash-separated and relative to the spec directory.
- Lines are sorted bytewise by path and use the format
  `<lowercase-hex><two spaces><path><LF>`.
- A release is valid only when every manifest verifies and a recursive
  byte-for-byte comparison of all six spec trees succeeds.

Any normative edit, including a fixture correction, requires a version change,
regenerated manifest, identical replication to all six repositories, JSON
validation, and cross-repository byte comparison.
