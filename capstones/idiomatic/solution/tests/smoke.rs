//! Compile-time boundary smoke test for the finished solution crate.
//!
//! Aliases the solution crate to `subject` and runs the shared boundary check,
//! which exists mainly to ensure the public surface stays wired together.

use idiomatic_indexer_solution as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn idiomatic_solution_exposes_the_shared_scaffold() {
    smoke_contract::assert_public_boundary();
}
