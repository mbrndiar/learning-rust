# 🔌 Module 10: Application Integration

Applications exchange data across boundaries where Rust's internal types and
invariants no longer apply automatically. Deserialization, files, environment
variables, sockets, and HTTP all require explicit validation and error handling.

## 🎯 Learning objectives

After this module, you should be able to serialize typed values with Serde,
distinguish syntactic decoding from domain validation, preserve unknown or
optional data intentionally, explain a basic HTTP exchange over TCP, and keep
wire formats outside core business logic.

## 🧾 Serde and JSON

[Serde](https://serde.rs/) derives conversions between Rust data and data
formats. [`serde_json`](https://docs.rs/serde_json/) handles JSON:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    active: bool,
}
```

Successful deserialization proves that required fields have compatible JSON
types. It does not prove that a string is non-empty, a number is in range, or
cross-field rules hold. Convert a wire type into a validated domain type at the
boundary.

Use `#[serde(deny_unknown_fields)]` when extra input likely indicates a mistake.
Avoid it when forward compatibility requires ignoring fields added by another
system. This is a contract decision, not a universal rule.

## 🌐 TCP and HTTP

TCP provides an ordered byte stream, not messages. A protocol defines framing:
where headers end, how body length is known, and what encoding applies. HTTP/1.1
uses a request/status line, headers, a blank line, and an optional body.

Production software should use maintained HTTP client/server libraries for
timeouts, limits, TLS, redirects, chunking, and malformed input. The lesson uses
a tiny localhost exchange only to reveal the boundary beneath those libraries.
It is not a production HTTP parser.

## 📚 Lessons

- `01_serde_json.rs` — wire types, validation, serialization, error reporting
- `02_tcp_and_http.rs` — localhost listener/client, framing, status and body

## ▶️ Running

```bash
cargo run --example lesson-10-serde-json
cargo run --example lesson-10-tcp-http
```

Then practice with
[`exercises/10_application_integration/`](../../exercises/10_application_integration/README.md).

## ⚠️ Common mistakes

- Treating decoded JSON as valid domain data.
- Using `serde_json::Value` everywhere when a stable typed schema exists.
- Losing a structured source error by replacing it with a vague string.
- Assuming one socket read equals one complete request.
- Omitting timeouts or input size limits in production networking.
- Hand-writing a production protocol implementation for a solved problem.

## ❓ Review questions

1. What does successful deserialization guarantee?
2. Why might a wire type differ from a domain type?
3. When is `deny_unknown_fields` helpful or harmful?
4. Why does TCP need application-level framing?
5. Which responsibilities are omitted from the teaching HTTP example?
