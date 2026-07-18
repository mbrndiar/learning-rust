//! Smoke test for the solution crate.
//!
//! Runs the shared boundary check against the built binaries to confirm the
//! completed package exposes the expected public surface and real adapters.

use std::path::Path;
use tasks_solution as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn tasks_solution_exposes_completed_shared_boundaries() {
    smoke_contract::assert_solution_public_boundary(
        Path::new(env!("CARGO_BIN_EXE_tasks-api")),
        Path::new(env!("CARGO_BIN_EXE_tasks")),
    );
}
