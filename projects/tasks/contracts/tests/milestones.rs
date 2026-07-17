//! Milestone contracts run against the finished solution.
//!
//! Unlike the starter's copy, none of these tests are `#[ignore]`d: the shared
//! `tests/contracts/milestones.rs` suite is expected to pass in full here,
//! confirming `tasks-solution` fulfills every milestone contract.

use tasks_solution as subject;

#[path = "../../tests/contracts/milestones.rs"]
mod contract;

#[test]
fn milestone_1_domain_and_contracts() {
    contract::milestone_1_domain_and_contracts();
}

#[test]
fn milestone_2_persistence() {
    contract::milestone_2_persistence();
}

#[test]
fn milestone_3_client_and_boundary() {
    contract::milestone_3_client_and_boundary();
}

#[test]
fn milestone_4_axum() {
    contract::milestone_4_axum();
}

#[test]
fn milestone_5_actix_and_interoperability() {
    contract::milestone_5_actix_and_interoperability();
}
