# 🦀⚡ Rust Cheat Sheet & Glossary

A compact companion to the lessons. Use it to recall syntax and ownership
choices after learning the concepts, not as a substitute for understanding
compiler diagnostics.

## 🗣️ Glossary

| Term | Meaning |
| --- | --- |
| Binding | A name associated with a value by `let` |
| Macro | Code-generating invocation marked with `!` (`println!`, `vec!`) |
| Attribute | Metadata in `#[...]` applied to the following item |
| Derive | An attribute asking the compiler/macro to generate trait implementations |
| Ownership | The rules determining which binding is responsible for a value |
| Move | Transfer of ownership for a non-`Copy` value |
| Borrow | Temporary access through `&T` or `&mut T` |
| Slice | Borrowed view into contiguous data (`&str`, `&[T]`) |
| Lifetime | The region for which a reference is valid |
| Struct | A product type combining fields |
| Enum | A sum type representing one of several variants |
| Pattern | Syntax that matches and destructures a value |
| Trait | A contract describing shared behavior |
| Generic | Code parameterized over a type, lifetime, or constant |
| Trait object | Dynamically dispatched erased type (`dyn Trait`) |
| Iterator | A lazy producer implementing `Iterator::next` |
| Closure | Anonymous callable that may capture its environment |
| `Option<T>` | Explicit presence (`Some`) or absence (`None`) |
| `Result<T, E>` | Explicit success (`Ok`) or recoverable failure (`Err`) |
| Panic | Unrecoverable failure or broken invariant that unwinds/aborts |
| RAII | Resource cleanup tied to a value's lifetime and `Drop` |
| Crate | Rust compilation unit: a library or binary |
| Package | A `Cargo.toml` and its crate targets |
| Workspace | Packages sharing dependency resolution and commands |
| Future | A value representing work that may complete later |
| Transaction | A group of database changes committed or rolled back together |
| Wire type | A type shaped for serialization at an external boundary |
| `Rc<T>` / `RefCell<T>` | Single-threaded shared ownership / runtime-checked mutation |
| `Send` | A type may transfer ownership across threads |
| `Sync` | Shared references to a type may be used across threads |

## 🔣 Bindings and core types

```rust
const MAX_RETRIES: u8 = 3;

let immutable = 42;
let mut mutable = 1;
mutable += 1;

let signed: i32 = -5;
let unsigned: u64 = 5;
let float: f64 = 3.14;
let enabled: bool = true;
let letter: char = '🦀';

let tuple: (&str, u8) = ("Ada", 36);
let array: [i32; 3] = [1, 2, 3];
```

Shadowing creates a new binding:

```rust
let spaces = "   ";
let spaces = spaces.len();
```

## 📝 Strings

```rust
let literal: &str = "borrowed UTF-8";
let mut owned = String::from("owned");
owned.push('!');
owned.push_str(" text");

fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

owned.len();            // bytes
owned.chars().count();  // Unicode scalar values
```

Rust strings do not support integer indexing. Use `.chars()`, `.bytes()`, or a
slice only at known UTF-8 boundaries.

## 🧩 Functions and expressions

```rust
fn area(width: u32, height: u32) -> u32 {
    width * height
}

let value = {
    let base = 20;
    base + 2
};
```

A final expression without `;` becomes the block's value. A semicolon turns it
into a statement producing `()`.

## 🚦 Control flow

```rust
let label = if score >= 60 { "pass" } else { "retry" };

match value {
    0 => println!("zero"),
    1..=9 => println!("single digit"),
    number if number % 2 == 0 => println!("even"),
    _ => println!("something else"),
}

for item in &items {
    println!("{item}");
}

let result = loop {
    if ready {
        break 42;
    }
};
```

## 🔐 Ownership and borrowing

```rust
let first = String::from("owned");
let second = first;          // move
let third = second.clone();  // explicit deep-enough duplicate

fn read(text: &str) -> usize {
    text.len()
}

fn change(text: &mut String) {
    text.push('!');
}
```

Borrowing rule: any number of shared references or one exclusive mutable
reference for the same data at a time. References must remain valid for every
use.

Common parameter choices:

| Need | Prefer |
| --- | --- |
| read text | `&str` |
| read a sequence | `&[T]` |
| mutate caller's value | `&mut T` |
| store or consume the value | `T`, `String`, `Vec<T>` |
| optional value | `Option<T>` |

## 🏗️ Structs, enums, and patterns

```rust
struct User {
    name: String,
    active: bool,
}

impl User {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            active: true,
        }
    }

    fn deactivate(&mut self) {
        self.active = false;
    }
}

enum Command {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}

match command {
    Command::Quit => stop(),
    Command::Move { x, y } => move_to(x, y),
    Command::Write(text) => println!("{text}"),
}
```

Concise single-pattern forms:

```rust
if let Some(value) = optional {
    println!("{value}");
}

let Some(value) = optional else {
    return;
};
```

## 🧺 Collections

```rust
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

let mut values = vec![1, 2, 3];
values.push(4);
values.get(10); // Option<&i32>

let mut counts = HashMap::new();
*counts.entry("rust").or_insert(0) += 1;

let unique: HashSet<_> = values.iter().copied().collect();
let ordered = BTreeMap::from([("a", 1), ("b", 2)]);
let queue = VecDeque::from([1, 2, 3]);
```

`HashMap`/`HashSet` order is unspecified. Use B-tree variants or sort output
when deterministic order is part of the behavior.

## ♻️ Iterators and closures

```rust
let squares: Vec<_> = values
    .iter()                    // &T
    .copied()                  // T when T: Copy
    .filter(|value| value % 2 == 0)
    .map(|value| value * value)
    .collect();

let total: i32 = values.iter().sum();
let found = values.iter().find(|value| **value > 10);

let factor = 3;
let scale = |value| value * factor;
```

| Entry | Item |
| --- | --- |
| `collection.iter()` | `&T` |
| `collection.iter_mut()` | `&mut T` |
| `collection.into_iter()` | `T` |

Adapters are lazy until consumed.

## 🛡️ `Option` and `Result`

```rust
fn parse_port(raw: &str) -> Result<u16, std::num::ParseIntError> {
    let port = raw.parse()?;
    Ok(port)
}

match result {
    Ok(value) => println!("{value}"),
    Err(error) => eprintln!("{error}"),
}

let value = optional.unwrap_or(default);
let value = optional.ok_or(MyError::Missing)?;
```

Use `unwrap`/`expect` only when failure is impossible by a demonstrated
invariant or appropriate for a short-lived test/example. At real input
boundaries, propagate or handle typed failure.

Error traits may be implemented manually or generated with `thiserror`:

```rust
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("missing item {0}")]
    Missing(u64),
    #[error("I/O failed")]
    Io(#[from] std::io::Error),
}
```

The attributes generate `Display`, `Error`, and the marked `From` conversion;
the enum still defines the application's structured failure cases.

## 🗂️ Modules and visibility

```rust
mod storage;

pub mod api {
    pub fn public() {}
    pub(crate) fn crate_only() {}
    fn private() {}
}

use std::path::{Path, PathBuf};
```

Items are private by default. `use` changes how a path is referenced, not who may
access it.

## 🧬 Generics and traits

```rust
trait Summary {
    fn summarize(&self) -> String;
}

fn announce(item: &impl Summary) {
    println!("{}", item.summarize());
}

fn largest<T: PartialOrd>(values: &[T]) -> Option<&T> {
    values
        .iter()
        .reduce(|left, right| if left >= right { left } else { right })
}

let heterogeneous: Vec<Box<dyn Summary>> = Vec::new();
```

Use generics/`impl Trait` for static dispatch and `dyn Trait` when values of
different concrete types must share one runtime collection or boundary.

## ⏳ Lifetimes

```rust
fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() { left } else { right }
}

struct Excerpt<'a> {
    text: &'a str,
}
```

Annotations describe reference relationships. They do not keep values alive or
change when they are dropped.

## 📁 Files and JSON

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Config {
    host: String,
    port: u16,
}

fn load(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let wire: Config = serde_json::from_str(&text)?;
    Ok(wire) // validate domain invariants before real use
}
```

Decoded structure still needs domain validation. File handles and lock guards
release automatically when dropped.

## 🗃️ SQL and SQLite

```rust
use rusqlite::{Connection, OptionalExtension, params};

let connection = Connection::open("app.sqlite")?;
connection.execute(
    "INSERT INTO notes (body) VALUES (?1)",
    params!["parameterized value"],
)?;

let body: Option<String> = connection
    .query_row(
        "SELECT body FROM notes WHERE id = ?1",
        [1],
        |row| row.get(0),
    )
    .optional()?;
```

Bind values rather than formatting SQL. Put invariants in constraints, map rows
explicitly, keep write transactions short, enable SQLite foreign keys per
connection, and use real temporary files in persistence tests.

## ⌨️ Terminal input

```rust
use std::io;

fn read_count() -> Result<u32, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().parse()?)
}
```

`read_line` keeps the newline. Trim and parse once at the input boundary, then
pass a typed value into core logic.

## 🧪 Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_case() {
        assert_eq!(double(4), 8);
    }

    #[test]
    fn failure_case() -> Result<(), MyError> {
        let value = fallible_operation()?;
        assert!(matches!(value, Expected::Variant));
        Ok(())
    }
}
```

Test normal behavior, boundaries, empty input, invalid input, and important
state transitions. Keep tests independent and deterministic.

## 🧵 Threads and shared state

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

let handle = thread::spawn(move || do_owned_work(data));
let result = handle.join().expect("worker should not panic");

let (sender, receiver) = mpsc::channel();
sender.send(value)?;
drop(sender); // closes channel when all sender clones are gone

let shared = Arc::new(Mutex::new(0));
let clone = Arc::clone(&shared);
```

Prefer clear ownership transfer. Keep mutex guards short and join workers so
panics are observed.

## 🌊 Async Rust

```rust
use tokio::time::{Duration, sleep, timeout};

async fn fetch() -> String {
    sleep(Duration::from_millis(10)).await;
    String::from("done")
}

#[tokio::main]
async fn main() {
    let (left, right) = tokio::join!(fetch(), fetch());

    let handle = tokio::spawn(async move { fetch().await });
    let value = handle.await.expect("task should not panic");

    let bounded = timeout(Duration::from_secs(1), fetch()).await;
}
```

Use async-aware I/O and sleep. Bound concurrency, observe task results, and do
not hold synchronous lock guards across `.await`.

## 🌐 JSON APIs and HTTP clients

```rust
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateRequest {
    name: String,
}

let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(2))
    .redirect(reqwest::redirect::Policy::none())
    .build()?;

let response: ResponseBody = client
    .get(format!("{base_url}/items"))
    .query(&[("name", "rust")])
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;
```

Decode wire types before validating domain values. Keep Axum or Actix Web
handlers thin, inject operations through state, map errors deliberately, check
status before response bodies, and shut local teaching servers down explicitly.

## 🛠️ Cargo command reference

```bash
rustc --version
cargo --version
rustup show active-toolchain

cargo new app
cargo add serde --features derive
cargo run
cargo run --example lesson-01-hello-world
cargo run -p idiomatic-indexer-solution --locked -- --help
cargo check --workspace --all-targets
cargo build --workspace
cargo build --workspace --release

cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings

cargo test
cargo test -p tasks-solution --locked
cargo test -p comparative-kv-solution --locked
cargo test -p idiomatic-indexer-solution --locked
cargo test test_name -- --nocapture
cargo test --doc --workspace --locked
cargo audit
cargo llvm-cov -p tasks-solution --all-targets --summary-only --fail-under-lines 85 --locked
cargo llvm-cov -p comparative-kv-solution --all-targets --summary-only --locked

cargo doc --workspace --no-deps --locked --open
cargo tree
cargo metadata --format-version 1 --locked --no-deps
cargo update
cargo clean

python3 scripts/check-markdown-links.py

rustc --explain E0382
```

Repository gates use `--locked` so `Cargo.lock` cannot change silently. Omit it
only when intentionally adding or updating dependencies, then review and commit
the lockfile change.

## 🚨 Common compiler diagnostics

| Code / message | Usually means | First question |
| --- | --- | --- |
| E0308 mismatched types | expression has another type | Which branch/argument defines the expected type? |
| E0382 use of moved value | ownership was transferred | Should this be borrowed, returned, or intentionally cloned? |
| E0499 multiple mutable borrows | exclusive borrows overlap | Can one borrow end earlier or data be partitioned? |
| E0502 mutable + immutable borrow | borrow modes overlap | Where is the shared reference last used? |
| E0515 returns reference to local | referent will be dropped | Should the function return owned data or borrow an input? |
| trait bound not satisfied | generic operation lacks required behavior | Is the bound right, or is the operation too demanding? |

Read the primary diagnostic, labels, notes, and help. Fix the design rule rather
than only the highlighted line.

## 🚀 Where to go next

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust standard library documentation](https://doc.rust-lang.org/std/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust Reference](https://doc.rust-lang.org/reference/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Serde](https://serde.rs/)
- [rusqlite](https://docs.rs/rusqlite/)
- [Axum](https://docs.rs/axum/)
- [Reqwest](https://docs.rs/reqwest/)
- [Actix Web](https://actix.rs/)
- [Tokio tutorial](https://tokio.rs/tokio/tutorial)
- [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)
- [`tempfile`](https://docs.rs/tempfile/)
- [`Rc<T>`](https://doc.rust-lang.org/std/rc/struct.Rc.html),
  [`RefCell<T>`](https://doc.rust-lang.org/std/cell/struct.RefCell.html), and
  interior mutability for single-threaded shared structures
- [Rustonomicon](https://doc.rust-lang.org/nomicon/) — only after mastering safe
  Rust and when unsafe code is genuinely required

Prefer documentation matching the compiler and crate versions selected by the
workspace. Check them with `rustc --version` and `cargo tree`.
