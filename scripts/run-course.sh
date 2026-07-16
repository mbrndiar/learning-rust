#!/usr/bin/env bash
set -euo pipefail

lessons=(
  lesson-01-hello-world
  lesson-01-variables-types
  lesson-01-strings-functions
  lesson-02-conditionals-match
  lesson-02-loops-ranges
  lesson-03-moves-clone
  lesson-03-references-slices
  lesson-04-structs-methods
  lesson-04-enums-patterns
  lesson-05-collections
  lesson-05-iterators-closures
  lesson-06-result-errors
  lesson-06-modules-files
  lesson-07-generics-traits
  lesson-07-lifetimes-trait-objects
  lesson-08-unit-tests
  lesson-08-test-design
  lesson-09-cargo-workflow
  lesson-09-diagnostics-cli
  lesson-10-schema-parameters
  lesson-10-crud-queries
  lesson-10-transactions-repositories
  lesson-11-threads-channels
  lesson-11-shared-state
  lesson-12-async-await
  lesson-12-concurrent-tasks
  lesson-13-serde-wire-domain
  lesson-13-axum-api
  lesson-13-reqwest-client
  lesson-13-actix-api
)

solutions=(
  solution-01-basics
  solution-02-control-flow
  solution-03-ownership
  solution-04-domain-types
  solution-05-collections
  solution-06-errors-io
  solution-07-traits-lifetimes
  solution-08-testing
  solution-09-tooling
  solution-10-sqlite
  solution-11-concurrency
  solution-12-async
  solution-13-rest-http
)

for example in "${lessons[@]}" "${solutions[@]}"; do
  printf '\n==> cargo run --quiet --locked --example %s\n' "$example"
  cargo run --quiet --locked --example "$example"
done

printf '\n==> testing lesson-specific test examples\n'
cargo test --quiet --locked --example lesson-08-unit-tests
cargo test --quiet --locked --example lesson-08-test-design
cargo test --quiet --locked --example lesson-09-diagnostics-cli

printf '\nAll lessons and reference solutions passed.\n'
