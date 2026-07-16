# 🌐🧩 Exercises: Module 13 — REST APIs and HTTP Clients

Complete one deliberately small Axum/Reqwest flow:

- strictly decode a label request and reject unknown fields;
- convert the wire type into a validated domain value;
- invoke one injected operation from a thin handler;
- configure a finite timeout and no-redirect client policy;
- check success status before decoding the response;
- reject one malformed local JSON response; and
- shut the ephemeral loopback server down deterministically.

Run:

```bash
cargo test --example ex-13-rest-http
cargo run --example solution-13-rest-http
```

This is not the Task project: there is no Task domain, persistence, list of
routes, or long-running server.

## 💡 Hint ladder

1. Deserialize into `CreateLabelRequest`, then call `Label::try_from`.
2. Put the operation behind `Arc<dyn LabelOperation>`.
3. Map validation failure to 422 and malformed JSON to the extractor's rejection.
4. Call `error_for_status()` before `.json()`.
5. Send the shutdown signal even after observing the malformed-body error.
