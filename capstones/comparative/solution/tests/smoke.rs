use comparative_kv_solution as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn comparative_solution_exposes_the_shared_boundary() {
    smoke_contract::assert_public_boundary();
}
