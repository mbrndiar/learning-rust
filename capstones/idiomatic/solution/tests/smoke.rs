use idiomatic_indexer_solution as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn idiomatic_solution_exposes_the_shared_scaffold() {
    smoke_contract::assert_public_boundary();
}
