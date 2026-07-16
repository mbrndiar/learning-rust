use std::path::Path;
use tasks_starter as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn tasks_scaffold_exposes_shared_boundaries_without_side_effects() {
    smoke_contract::assert_public_boundary(
        Path::new(env!("CARGO_BIN_EXE_tasks-api")),
        Path::new(env!("CARGO_BIN_EXE_tasks")),
    );
}
