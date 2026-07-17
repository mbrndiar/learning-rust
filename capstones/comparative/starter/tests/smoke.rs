//! Compile-time boundary smoke test for the starter scaffold crate.
//!
//! Aliases the starter crate to `subject` and runs the shared boundary check, which
//! confirms the scaffold keeps the same public surface as the solution.

use comparative_kv_starter as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn comparative_starter_exposes_the_shared_scaffold() {
    smoke_contract::assert_public_boundary();
}
