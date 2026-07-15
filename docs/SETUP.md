# 🛠️🦀 Setting Up Your Rust Environment

Rust ships as a coordinated toolchain: the compiler (`rustc`), package manager
and build tool (`cargo`), formatter (`rustfmt`), linter (`clippy`), documentation
tool (`rustdoc`), and standard library.

## 🦀 1. Install Rust with rustup

Follow the official instructions at <https://rustup.rs/>. On Linux and macOS the
installer command currently shown there is:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows, download and run `rustup-init.exe`. Rust may require a C linker; the
installer explains the platform-specific prerequisite when one is missing.

Restart the terminal, then verify the installation:

```bash
rustc --version
cargo --version
rustup show active-toolchain
```

This course requires Rust 1.85 or newer because it uses edition 2024.

## 📥 2. Get the code

```bash
git clone https://github.com/mbrndiar/learning-rust.git
cd learning-rust
```

## 🧰 3. Install required components

The default installation profile usually includes the formatter and linter.
Installing them explicitly is safe:

```bash
rustup component add rustfmt clippy
```

Use stable Rust unless a lesson explicitly discusses another channel:

```bash
rustup default stable
rustup update stable
```

`rust-toolchain.toml` files can pin a project-specific channel. This repository
instead declares its minimum compiler in `Cargo.toml` and lets stable advance.

## 🧑‍💻 4. Choose an editor

Any editor works. A common setup is:

- [Visual Studio Code](https://code.visualstudio.com/) with the
  [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
  extension; or
- [RustRover](https://www.jetbrains.com/rust/) from JetBrains.

rust-analyzer uses the same Cargo metadata as the terminal. If editor diagnostics
disagree with Cargo, first confirm that both use the same workspace and toolchain.

## 🚀 5. Build and run the first lesson

```bash
cargo run --example lesson-01-hello-world
```

The first build downloads dependencies and compiles them. Later builds reuse
artifacts from `target/`.

## 🗂️ 6. Understand Cargo files

- `Cargo.toml` declares packages, workspace members, direct dependencies,
  targets, and configuration.
- `Cargo.lock` records exact dependency versions selected for this application
  workspace and should be committed.
- `src/lib.rs`, `src/main.rs`, `examples/`, `tests/`, and `benches/` are Cargo's
  conventional target locations. This course explicitly lists teaching examples
  because they are grouped into module directories.
- `target/` contains generated artifacts and should not be committed.

## 🏗️ 7. Create your own Cargo project

The course uses a workspace with explicitly named examples. Ordinary
applications start with a simpler generated package:

```bash
cargo new hello-rust
cd hello-rust
cargo run
```

Cargo creates:

```text
hello-rust/
├── Cargo.toml
└── src/
    └── main.rs
```

Edit `src/main.rs`; `fn main()` is the binary entry point. `cargo check`
type-checks quickly, while `cargo run` builds and executes the program.

When a project needs a third-party crate, prefer Cargo's dependency command:

```bash
cargo add serde --features derive
```

This updates `Cargo.toml`; the next Cargo command resolves the dependency and
updates `Cargo.lock`. Add a dependency only when the program uses it, and read
the crate's documentation for supported features and minimum Rust version.

The [`Beginner's Guide`](BEGINNER_GUIDE.md) explains how packages, crates,
targets, examples, and this workspace relate.

## 🔄 Daily development flow

Start narrow, then widen:

```bash
# Run the behavior you changed.
cargo test -p task-manager storage

# Apply canonical formatting.
cargo fmt --all

# Ask Clippy for static feedback and reject warnings.
cargo clippy --workspace --all-targets -- -D warnings

# Run application tests and documentation examples.
cargo test --workspace --lib --bins
cargo test -p task-manager
cargo test --doc --workspace

# Compile all teaching targets, including unfinished exercise starters.
cargo check --workspace --all-targets
```

`cargo fmt` changes files. CI uses `cargo fmt --all --check` to verify that the
committed result is already formatted. Clippy suggestions are context-sensitive:
understand ownership and behavior before accepting an automated rewrite.

## 📊 Optional coverage tool

The CI workflow reports capstone test coverage with
[`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov). To run the same
summary locally:

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
cargo llvm-cov -p task-manager --all-targets --summary-only --locked
```

This is a diagnostic report, not a correctness score or a required percentage.

## 🩺 Troubleshooting

### 🚫 `cargo` is not found

Restart the shell after installing rustup. On Unix-like systems, confirm that
`$HOME/.cargo/bin` is on `PATH`. The installer normally adds it to your shell
profile.

### ⏳ The compiler is too old

```bash
rustup update stable
rustup default stable
```

Then check `rustc --version`. If a directory has a pinned override, inspect it
with `rustup show` and remove it only when you understand why it exists.

### 🔗 A native dependency fails to link

Read the first linker error and install the platform build tools it names. On
Linux this commonly means a C compiler and development headers; on Windows it
often means Visual Studio Build Tools.

### 🧹 The build cache is stale or very large

`cargo clean` deletes generated artifacts for the current workspace. It is safe
but usually unnecessary and makes the next build slower.

### 🌐 A dependency download fails

Retry after checking network and proxy configuration. Cargo's registry and Git
settings live under `$CARGO_HOME` (normally `~/.cargo`); do not commit personal
credentials or machine-specific configuration.
