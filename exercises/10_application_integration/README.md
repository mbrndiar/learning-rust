# Exercises: Module 10 — Application Integration

Deserialize `ServerInput`, then validate it into `ServerConfig`:

- trim and reject an empty host;
- require a nonzero port;
- default `workers` to one when omitted;
- reject zero workers;
- serialize the validated domain value.

Run:

```bash
cargo test --example ex-10-integration
cargo run --example solution-10-integration
```

Keep JSON syntax/type errors distinct from domain validation errors.

## Hints

1. Deserialize into `ServerInput` first; do not validate raw JSON manually.
2. Use `map_err` to prefix syntax/type failures with `"invalid JSON"`.
3. `Option::unwrap_or(1)` applies the documented worker default.
4. Validate trimmed values before constructing `ServerConfig`.
