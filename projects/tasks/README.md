# Task REST API and Rust client

Build one Task application behind two native async HTTP server adapters—Axum and
Actix Web—and call either server through one Reqwest client and shared `tasks`
CLI. SQLite and a deterministic, versioned Markdown checklist implement the same
repository contract. The goal is one domain and one observable contract, not a
framework-shaped domain or four unrelated applications.

This required project belongs after
[Module 13](../../lessons/13_rest_apis_and_http_clients/README.md) and before the final
[`capstones`](../../capstones/README.md). The solution completes the shared core,
both repositories, strict HTTP/client boundary, Reqwest CLI, and native Axum and
Actix Web lifecycles.

## Start with the portable contract

- [`docs/SPEC.md`](docs/SPEC.md) defines Task, persistence, HTTP, error, and CLI
  behavior.
- [`docs/openapi.yaml`](docs/openapi.yaml) is the byte-identical portable OpenAPI
  3.1 contract shared with the Go and Python courses.
- [`docs/PLAN.md`](docs/PLAN.md) is the reusable ecosystem-neutral adaptation
  plan.
- [`docs/PROMPT.md`](docs/PROMPT.md) is the reusable agent prompt.

## Rust architecture

The starter and solution expose the same public modules and executable names;
the small `contracts` package runs the completed shared milestone suite:

```text
projects/tasks/
├── contracts/                     executable shared solution contracts
└── {starter,solution}/
    ├── src/
    │   ├── domain.rs                 Task values and update/filter inputs
    │   ├── error.rs                  typed TaskError boundary
    │   ├── application.rs            repository trait and application service
    │   ├── storage/{sqlite,markdown}.rs
    │   ├── api/{boundary,axum,actix}.rs
    │   ├── client.rs                 Reqwest transport boundary
    │   ├── cli.rs                    shared command policy
    │   ├── server.rs                 backend/server selection and lifecycle
    │   └── bin/{tasks-api,tasks}.rs  thin composition roots
    └── tests/                        shared smoke and milestone wrappers
```

Dependencies point inward. `domain`, `error`, and `application` do not depend on
web frameworks or Reqwest. Storage adapters implement the repository trait. API
adapters translate inbound HTTP at the boundary. The Reqwest client knows only
the portable HTTP contract. `server` and the two binaries are composition roots.
Axum and Actix Web remain separate adapters so their native routing, extraction,
state, response, and lifecycle patterns stay visible.

## Five milestones

1. **Domain and contracts** — validated Task values, typed errors, repository
   trait, application service, and boundary models.
2. **Persistence** — interchangeable SQLite and versioned Markdown repositories
   passing one shared contract.
3. **Client and HTTP boundary** — Reqwest transport, CLI policy, portable JSON and
   error mapping, timeouts, and response validation.
4. **Axum server** — thin Axum routes, injected state, loopback lifecycle, and the
   shared black-box HTTP contract.
5. **Actix Web and interoperability** — thin Actix Web routes and the complete
   one-client-by-two-server matrix, including OpenAPI comparison.

Attempt each ignored starter milestone before consulting the corresponding
solution work. All five solution milestones are active; the starter milestones
remain ignored exercises.

## Exact commands

Run from the repository root:

```bash
cargo metadata --format-version 1 --locked --no-deps
cargo check -p tasks-starter --all-targets --locked
cargo check -p tasks-solution --all-targets --locked
cargo test -p tasks-contracts --locked
cargo test -p tasks-starter --locked
cargo test -p tasks-solution --locked
cargo test -p tasks-starter milestone_1_domain_and_contracts -- --ignored
cargo test -p tasks-solution --test milestones
cargo test -p tasks-solution --test http_contracts

cargo run -p tasks-starter --bin tasks-api-starter -- --help
cargo run -p tasks-starter --bin tasks-starter -- --help
cargo run -p tasks-solution --bin tasks-api -- --help
cargo run -p tasks-solution --bin tasks -- --help
cargo run -p tasks-solution --bin tasks-api -- \
  --server axum --backend sqlite --data tasks.db
cargo run -p tasks-solution --bin tasks-api -- \
  --server actix --backend markdown --data tasks.md

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --lib --bins --locked
cargo test -p comparative-kv-solution --locked
cargo test -p idiomatic-indexer-solution --locked
cargo test --doc --workspace --locked
cargo doc --workspace --no-deps --locked
python3 scripts/check-markdown-links.py
```

An ignored starter milestone is expected to fail until implemented. Starter
executables exit visibly with a typed `TaskError::Incomplete` and do not create
storage. The solution accepts `--server axum|actix` with either
`--backend sqlite|markdown`.

## Dependency and MSRV proof

The workspace remains Rust 2024 with resolver 3 and `rust-version = "1.85"`.
Before adding the packages, a minimal Rust 1.85 compile imported and type-checked
exact Axum `0.8.9`, Actix Web `4.12.1`, and Reqwest `0.13.4`. Resolver 3 selected
Rust-1.85-compatible transitives, notably `actix-http 3.11.2`, `time 0.3.45`, and
`bytestring 1.5.0`; no extra compatibility override was needed. Actix Web is
pinned because `4.14.0` requires Rust 1.88. The checked-in lockfile is the final
authority and both stable and Rust 1.85 gates use `--locked`.

Features are deliberately narrow: Axum uses HTTP/1, JSON, query, and Tokio; Actix
Web disables default compression and cookie features; Reqwest disables its
default TLS stack and enables JSON and query only. Project traffic is loopback HTTP. No selected
stack enables WebSockets, multipart, cookies, compression, automatic retries, or
production TLS/deployment facilities.

## What the comparison should reveal

| Stack | Makes visible | Provides |
| --- | --- | --- |
| Axum | extractors, typed state, router composition, response conversion | Tower/Hyper-oriented async HTTP ergonomics |
| Actix Web | application data, scopes, handlers, responders, server lifecycle | Actix-native runtime and test patterns |
| Reqwest | URL/request construction, status-first decoding, timeouts, transport errors | one reusable asynchronous HTTP client API |
| rusqlite | statements, row mapping, transactions, connection ownership | direct SQLite access with bundled SQLite |
| Markdown | parsing untrusted text, deterministic serialization, atomic replacement | a human-readable single-file backend |

Both native servers call the same runtime-neutral `TaskApplication`, which moves
synchronous repository work onto Tokio's blocking pool. Axum owns its Tokio
listener directly. Actix owns a standard listener and runs `HttpServer` inside a
dedicated Actix `System`; `web::Data` shares the application boundary with each
worker. The same strict Reqwest client passes all four server/backend
combinations, including persistence across restart and monotonic IDs.

The trade-off is a little lifecycle code to preserve each framework's native
runtime and HTTP types. In return, the domain, repositories, JSON policy, and
client stay framework-neutral without hiding either server behind a universal
router.

## Educational boundaries

Servers bind only to loopback in examples and tests use ephemeral ports, finite
timeouts, temporary storage, and no public network. This project is not production
deployment guidance. It intentionally omits authentication, browser UI, CORS,
WebSockets, streaming, multipart, cookies, compression, TLS termination,
containers, ORM/migrations, distributed transactions, automatic retries,
cross-process Markdown locking, generated SDKs, and operational hardening. See
[`docs/SPEC.md`](docs/SPEC.md#explicit-non-goals) for the behavioral boundary.
