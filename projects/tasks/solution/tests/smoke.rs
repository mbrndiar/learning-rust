//! Smoke test for the solution crate.
//!
//! Runs the shared boundary check against the built binaries to confirm the
//! scaffold exposes the expected public surface without side effects.

use std::path::Path;
use tasks_solution as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn tasks_scaffold_exposes_shared_boundaries_without_side_effects() {
    smoke_contract::assert_solution_public_boundary(
        Path::new(env!("CARGO_BIN_EXE_tasks-api")),
        Path::new(env!("CARGO_BIN_EXE_tasks")),
    );
}
