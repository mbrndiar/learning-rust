//! Milestone tests for the starter crate.
//!
//! A thin wrapper over the shared `tests/contracts/milestones.rs` suite. Each
//! test is `#[ignore]`d with a TODO note; remove the attribute as you complete
//! the matching milestone so the shared contract runs against your code.

use tasks_starter as subject;

#[path = "../../tests/contracts/milestones.rs"]
mod contract;

// Forces a compile-time reference to the crate under test so the wrapper fails
// to build if the public surface is missing, before any milestone runs.
fn assert_subject_is_linked() {
    let _ = subject::TaskFilter::default();
}

#[test]
#[ignore = "TODO: implement milestone 1 domain and application contracts"]
fn milestone_1_domain_and_contracts() {
    assert_subject_is_linked();
    contract::milestone_1_domain_and_contracts();
}

#[test]
#[ignore = "TODO: implement both persistence adapters"]
fn milestone_2_persistence() {
    assert_subject_is_linked();
    contract::milestone_2_persistence();
}

#[test]
#[ignore = "TODO: implement Reqwest and shared HTTP boundaries"]
fn milestone_3_client_and_boundary() {
    assert_subject_is_linked();
    contract::milestone_3_client_and_boundary();
}

#[test]
#[ignore = "TODO: implement Axum"]
fn milestone_4_axum() {
    assert_subject_is_linked();
    contract::milestone_4_axum();
}

#[test]
#[ignore = "TODO: implement Actix Web and interoperability"]
fn milestone_5_actix_and_interoperability() {
    assert_subject_is_linked();
    contract::milestone_5_actix_and_interoperability();
}
