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
