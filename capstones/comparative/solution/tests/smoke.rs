//! Compile-time boundary smoke test for the finished solution crate.
//!
//! Aliases the solution crate to `subject` and runs the shared boundary check, which
//! mainly guarantees the public surface stays wired together and compilable.

use comparative_kv_solution as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn comparative_solution_exposes_the_shared_boundary() {
    smoke_contract::assert_public_boundary();
}
