//! Runs the shared milestone contract against the finished solution crate.
//!
//! The crate under test is aliased to `subject`, and the single contract file is
//! included with `#[path]` so the identical suite compiles against both the starter
//! and the solution. Every milestone is active here because the solution is
//! complete.

use idiomatic_indexer_solution as subject;
use std::path::Path;

#[path = "../../tests/contracts/milestones.rs"]
mod contract;

fn program() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_idiomatic-indexer-solution"))
}

#[test]
fn milestone_1_validated_domain() {
    contract::milestone_1_validated_domain();
}

#[test]
fn milestone_2_traversal_and_issues() {
    contract::milestone_2_traversal_and_issues();
}

#[test]
fn milestone_3_versioned_storage() {
    contract::milestone_3_versioned_storage();
}

#[test]
fn milestone_4_bounded_concurrency() {
    contract::milestone_4_bounded_concurrency();
}

#[test]
fn milestone_5_full_cli() {
    contract::milestone_5_full_cli(program());
}
