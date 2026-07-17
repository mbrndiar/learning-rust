//! Compile-time boundary smoke test for the guided starter crate.
//!
//! Aliases the starter crate to `subject` and runs the shared boundary check, which
//! verifies the public surface stays wired together even before the milestone
//! bodies are implemented.

use idiomatic_indexer_starter as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn idiomatic_starter_exposes_the_shared_scaffold() {
    smoke_contract::assert_public_boundary();
}
