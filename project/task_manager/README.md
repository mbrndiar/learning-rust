# ✅📋 Capstone: Task Manager

Task Manager is a file-backed CLI that combines ownership, domain types, traits,
dependency injection, typed errors, Serde, atomic persistence, Clap, and tests.

## 🏛️ Architecture

```text
main
  -> Cli (Clap parsing)
  -> execute(command)
  -> TaskManager<S: TaskStore>
       |-- InMemoryTaskStore
       `-- JsonFileTaskStore -> versioned JSON envelope
```

- `domain.rs` owns `TaskId`, `Task`, `TaskError`, the `TaskStore` contract, and
  storage-independent `TaskManager`.
- `storage.rs` implements an in-memory strategy and a JSON file strategy.
- `cli.rs` converts parsed commands into domain operations and formatted output.
- `main.rs` owns terminal output and process exit status.

The domain layer does not parse arguments, choose paths, print, or serialize.
The JSON strategy validates deserialized data because Serde construction alone
cannot enforce nonzero IDs, non-empty titles, uniqueness, or `next_id`.

## 🧭 Guided code tour

Read the project in dependency order rather than starting with the longest file:

1. **`src/main.rs`:** identify the process boundary—parse arguments, call the
   library, print output or an error, choose an exit code.
2. **`src/cli.rs`:** follow one `Command` variant through `execute`; notice that
   formatting is separate from storage.
3. **`src/domain.rs` — `TaskId` and `Task`:** list the invariants enforced by
   constructors and methods. Fields stay private so callers cannot create an
   empty title or mutate completion arbitrarily.
4. **`src/domain.rs` — `TaskStore` and `TaskManager`:** translate each trait
   method into a storage responsibility, then identify the filtering that
   remains domain logic.
5. **`src/storage.rs` — `InMemoryTaskStore`:** understand the simplest complete
   implementation before reading file I/O.
6. **`src/storage.rs` — `JsonFileTaskStore`:** trace load, validation, candidate
   mutation, temporary-file write, and commit.
7. **`tests/task_manager.rs`:** see the same behavior contract applied to both
   storage strategies.

`TaskId` has a hand-written `Deserialize` implementation. A derived
implementation would construct the private wrapper directly and could accept
JSON `0`, bypassing `TaskId::new`. The custom implementation routes external
data through the same nonzero invariant.

To rebuild the design as an assessment, start with `Task` plus
`InMemoryTaskStore`, add `TaskManager`, write the command-independent tests, and
only then add Clap and JSON persistence.

## 🚀 Use the CLI

From the repository root:

```bash
cargo run -p task-manager -- add "Learn ownership"
cargo run -p task-manager -- add "Practice traits"
cargo run -p task-manager -- list
cargo run -p task-manager -- complete 1
cargo run -p task-manager -- list --pending-only
cargo run -p task-manager -- remove 1
```

The default file is `tasks.json` in the current directory. Put global options
before the subcommand to select another path:

```bash
cargo run -p task-manager -- --storage /tmp/course-tasks.json add "Temporary task"
```

Use `--help` at either level:

```bash
cargo run -p task-manager -- --help
cargo run -p task-manager -- list --help
```

## 💾🔒 Persistence guarantees

The file contains a versioned object with `next_id` and `tasks`. IDs are not
reused after deletion. On load, the backend rejects:

- unsupported storage versions;
- zero or duplicate task IDs;
- blank titles;
- a zero `next_id`; or
- `next_id` that does not exceed every stored ID.

Each mutation is transactional in memory: it clones the current state, applies
the operation, writes the candidate through a temporary file, synchronizes it,
and persists it over the destination before replacing live state. A failed save
therefore does not report success or leave the in-process state ahead of disk.

## 🧪 Test

```bash
cargo test -p task-manager
```

The tests apply one behavior contract to both storage implementations, verify
round-trip persistence and monotonic IDs, reject inconsistent JSON, isolate
files in temporary directories, and exercise command output without spawning a
process.

## 🎓 Learning checklist

- Trace `complete 1` from parsed command to persisted bytes.
- Explain why `TaskManager` is generic over `TaskStore`.
- Identify every location where input becomes a stronger domain type.
- Explain why file mutations use a candidate state.
- Add another store without modifying domain operations.

## 🧗 Extension exercises

Implement one change at a time and add a test before or alongside it:

1. Add an `edit ID TITLE` command without exposing mutable task fields.
2. Add a `Priority` enum and deterministic sorting.
3. Add due dates using a well-maintained date/time crate.
4. Add `list --contains TEXT` while keeping filtering in the domain layer.
5. Add JSON export to a separate destination without changing storage format.
6. Add an async HTTP adapter implementing the same domain-facing operations.
7. Add file locking and document the multi-process consistency model.
8. Add migration from storage version 1 to a richer version 2.

For every extension, define normal behavior, a boundary, a failure case, and
whether old files remain compatible.
