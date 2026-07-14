# 🏆 Capstone Project

[`task_manager/`](task_manager/README.md) is a complete command-line application
that combines the course:

```text
Clap CLI -> command execution -> TaskManager -> TaskStore trait
                                         |-- InMemoryTaskStore
                                         `-- JsonFileTaskStore -> atomic JSON file
```

Run all project tests:

```bash
cargo test -p task-manager
```

The implementation is intentionally small enough to read end-to-end while still
preserving domain invariants, structured errors, persistence safety, and
testability.
