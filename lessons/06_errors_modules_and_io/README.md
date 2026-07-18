# 🛡️📁 Module 6: Errors, Modules, and I/O

Rust distinguishes recoverable failure (`Result`) from invariant violations and
unrecoverable states (panics). Modules organize visibility; ownership and RAII
make file cleanup deterministic.

## 🎯 Learning objectives

After this module, you should be able to return and propagate `Result`, use `?`,
perform checked numeric conversions, add context with custom error types, decide
when a panic is justified, organize module visibility, and read or write files
without leaking resources.

## 🛡️ Recoverable errors

`Result<T, E>` is either `Ok(T)` or `Err(E)`. The `?` operator returns early on
error and converts the error with `From` when necessary:

```rust
fn load(path: &Path) -> io::Result<String> {
    let text = fs::read_to_string(path)?;
    Ok(text)
}
```

Do not use `unwrap()` at a real input boundary. Match errors you can handle;
propagate errors that the caller is better positioned to interpret. A custom
enum can preserve whether parsing, I/O, validation, or a domain lookup failed.

Use `panic!` for broken internal invariants or impossible states, not for a
missing user file or malformed command-line value.

`From` represents conversions that cannot lose information. `TryFrom` and
`TryInto` return `Result` when a value may not fit, such as narrowing `u32` to
`u16`. Prefer that explicit failure over an `as` cast when truncation would
violate the domain contract.

The lesson implements `Display`, `Error`, and `From` manually so every layer is
visible. Application code often uses
[`thiserror`](https://docs.rs/thiserror/) to generate the same boilerplate:

```rust
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("invalid port: {0}")]
    InvalidPort(u16),
    #[error("cannot read configuration")]
    Io(#[from] std::io::Error),
}
```

The capstone's `TaskError` uses this approach. `thiserror` does not change
runtime error semantics; it generates the trait implementations described here.

## 🗂️ Modules and visibility

Modules create namespaces and privacy boundaries. Items are private by default.
`pub` exposes an item to callers; narrower forms such as `pub(crate)` avoid
making implementation details part of a public API. `use` brings paths into
scope but does not change visibility.

## 📁 Files and RAII

`Path` is a borrowed filesystem path; `PathBuf` owns one. File handles close
automatically when dropped, including during early returns. Buffered readers and
writers reduce system calls for incremental I/O.

Persistent text should declare UTF-8 behavior. Writes that must not leave
partial state should use a temporary file followed by a rename, as demonstrated
in the capstone. The lesson uses [`tempfile`](https://docs.rs/tempfile/) to
create a unique temporary directory whose RAII guard removes it automatically.

## 📘 Lessons

- `01_result_and_custom_errors.rs` — typed failure, `?`, `From`, error sources
- `02_modules_files_and_paths.rs` — nested modules, visibility, paths, buffered
  text I/O, cleanup

## 🚀 Running

```bash
cargo run --example lesson-06-result-errors
cargo run --example lesson-06-modules-files
```

Then practice with
[`exercises/06_errors_modules_and_io/`](../../exercises/06_errors_modules_and_io/README.md).

## 🚧 Common mistakes

- Using `unwrap()` because propagation initially requires more types.
- Returning `Option` when callers need to know why an operation failed.
- Converting every error to a string and losing structured causes.
- Catching an error only to print it and return success.
- Making every module item `pub`.
- Assuming the current working directory is the source file's directory.
- Using an error-derive crate without understanding which `Display`, `Error`,
  and conversion implementations it generates.

## 🧠 Review questions

1. How do `Option<T>` and `Result<T, E>` communicate different contracts?
2. What exactly does `?` do on `Err`?
3. Why is `TryFrom` preferable to `as` for checked narrowing?
4. When should a custom error preserve a source error?
5. Why are module items private by default?
6. How does RAII guarantee that a file is closed?
