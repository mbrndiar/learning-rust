# 📦 Retained Reference Project

[`task_manager/`](task_manager/README.md) is a complete command-line application
that combines the course foundations:

```text
Clap CLI -> command execution -> TaskManager -> TaskStore trait
                                         |-- InMemoryTaskStore
                                         `-- JsonFileTaskStore -> atomic JSON file
```

Run all project tests:

```bash
cargo test -p task-manager --locked
```

The implementation is intentionally small enough to read end-to-end while still
preserving domain invariants, structured errors, persistence safety, and
testability.

The primary assessment projects live under [`capstones/`](../capstones/README.md).
Task Manager is a workspace member for side-by-side study, tests, documentation,
and coverage. See the
[`old-to-new concept map`](../capstones/MIGRATION.md) before moving its design
lessons into either capstone.
