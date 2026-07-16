# 🌐🦀 Module 13: REST APIs and HTTP Clients

After async Rust, this module creates and invokes real HTTP APIs with maintained
libraries. Every example binds only to loopback on an operating-system-assigned
port, uses finite client timeouts, shuts down explicitly, and exits. No lesson
contacts the public network or leaves a server running.

## 🎯 Learning objectives

After this module, you should be able to define Serde wire types and convert them
into strict domain values; distinguish JSON syntax/type failures from validation
failures; build Axum routes with extractors, state, and error mapping; construct
Reqwest URLs and query parameters with timeout and redirect policies; validate
status before decoding a body; compare Actix Web's native routing, state, error,
and lifecycle model; and move blocking work to `spawn_blocking`.

## 🧾 Wire types and domain types

Deserialization checks JSON syntax and the declared wire shape. It does not prove
domain rules such as non-empty text or cross-field relationships. Decode into a
wire type, then use `TryFrom` to construct a validated domain value.
`#[serde(deny_unknown_fields)]` is useful for strict request contracts, but may be
wrong for formats that intentionally permit forward-compatible fields.

## 🧭 Server and client boundaries

Axum and Actix Web both provide routing, extraction, shared state, response
conversion, and lifecycle support, but their native APIs differ. Keep handlers
thin: extract, validate, invoke an injected operation, and map its result.

Reqwest should receive a finite timeout and an intentional redirect policy.
Construct query strings with `.query(...)`, not string concatenation. Check the
HTTP status before decoding the body so a server error is not mistaken for a
successful representation with an unexpected shape.

Blocking database, filesystem, or CPU work must not run directly on an async
worker. Use an async-aware API or deliberately call `spawn_blocking`, while
accounting for cancellation and shutdown.

## 📘 Lessons

- `01_serde_wire_and_domain.rs` — strict wire shapes, syntax/type errors,
  validation, unknown fields
- `02_axum_routes_state_and_errors.rs` — native routing, extractors, state, typed
  responses, graceful shutdown
- `03_reqwest_clients.rs` — URL/query construction, timeout, redirects,
  status-first decoding, malformed local response
- `04_actix_web_comparison.rs` — native Actix routing/state/errors,
  `spawn_blocking` guidance, lifecycle comparison

## 🚀 Running

```bash
cargo run --example lesson-13-serde-wire-domain
cargo run --example lesson-13-axum-api
cargo run --example lesson-13-reqwest-client
cargo run --example lesson-13-actix-api
```

Then practice with
[`exercises/13_rest_apis_and_http_clients/`](../../exercises/13_rest_apis_and_http_clients/README.md).
The larger Task project comes afterward; this module does not implement its
domain or persistence behavior.

## 🚧 Common mistakes

- Treating successfully decoded JSON as a validated domain value.
- Using one type for both a changing wire contract and internal invariants.
- Flattening every failure into status 500 or an unstructured string.
- Building URLs or query strings with manual concatenation.
- Decoding a body before checking whether the status represents success.
- Omitting timeouts or following redirects without an explicit policy.
- Running blocking SQLite/filesystem work directly in an async handler.
- Starting a server without retaining a shutdown handle and awaiting termination.

## 🧠 Review questions

1. How do JSON syntax/type errors differ from domain validation failures?
2. When should unknown JSON fields be rejected?
3. Which responsibilities belong in a thin handler?
4. Why should a client validate status before decoding the body?
5. What do timeout and redirect policies protect?
6. How do Axum and Actix Web express shared state differently?
7. When is `spawn_blocking` appropriate, and what does it not solve?
