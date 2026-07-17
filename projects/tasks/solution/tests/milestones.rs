//! Milestone tests for the solution crate.
//!
//! This is a thin wrapper: the real assertions live in the shared
//! `tests/contracts/milestones.rs` suite, included here against the solution
//! crate so the same contract runs against every implementation.

use tasks_solution as subject;

#[path = "../../tests/contracts/milestones.rs"]
mod contract;

// Forces a compile-time reference to the crate under test so the wrapper fails
// to build if the public surface is missing, before any milestone runs.
fn assert_subject_is_linked() {
    let _ = subject::TaskFilter::default();
}

#[test]
fn milestone_1_domain_and_contracts() {
    assert_subject_is_linked();
    contract::milestone_1_domain_and_contracts();
}

#[test]
fn milestone_2_persistence() {
    assert_subject_is_linked();
    contract::milestone_2_persistence();
}

#[test]
fn milestone_3_client_and_boundary() {
    assert_subject_is_linked();
    contract::milestone_3_client_and_boundary();
}

#[test]
fn milestone_4_axum() {
    assert_subject_is_linked();
    contract::milestone_4_axum();
}

#[test]
fn milestone_5_actix_and_interoperability() {
    assert_subject_is_linked();
    contract::milestone_5_actix_and_interoperability();
}
