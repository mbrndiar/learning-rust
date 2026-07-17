//! Smoke test for the starter crate.
//!
//! Runs the shared starter boundary check against the built binaries to confirm
//! the scaffold exposes the expected public surface without side effects, even
//! before any milestone is implemented.

use std::path::Path;
use tasks_starter as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn tasks_scaffold_exposes_shared_boundaries_without_side_effects() {
    smoke_contract::assert_starter_public_boundary(
        Path::new(env!("CARGO_BIN_EXE_tasks-api-starter")),
        Path::new(env!("CARGO_BIN_EXE_tasks-starter")),
    );
}
