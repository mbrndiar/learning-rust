use tasks_solution as subject;

#[path = "../../tests/contracts/milestones.rs"]
mod contract;

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
#[ignore = "Phase 1 scaffold: implement Actix Web and interoperability"]
fn milestone_5_actix_and_interoperability() {
    assert_subject_is_linked();
    contract::milestone_5_actix_and_interoperability();
}
