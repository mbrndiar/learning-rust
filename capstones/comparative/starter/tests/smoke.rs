use comparative_kv_starter as subject;

#[path = "../../tests/contracts/smoke.rs"]
mod smoke_contract;

#[test]
fn comparative_starter_exposes_the_shared_scaffold() {
    smoke_contract::assert_public_boundary();
}
