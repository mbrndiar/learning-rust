use comparative_kv_solution as subject;
use std::path::Path;

#[path = "../../tests/contracts/conformance.rs"]
mod contract;

fn program() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_comparative-kv-solution"))
}

#[test]
fn milestone_1_domain_fixtures() {
    contract::milestone_1_domain_fixtures();
}

#[test]
fn milestone_2_cli_and_invalid() {
    contract::milestone_2_cli_and_invalid(program());
}

#[test]
fn milestone_3_storage_and_migration() {
    contract::milestone_3_storage_and_migration(program());
}

#[test]
fn milestone_4_boundaries_and_mutations() {
    contract::milestone_4_boundaries_and_mutations(program());
}

#[test]
fn milestone_5_multiprocess() {
    contract::milestone_5_multiprocess(program());
}

#[test]
#[ignore = "subprocess entry point"]
fn conformance_actor_process() {
    contract::actor_process();
}

#[test]
#[ignore = "subprocess entry point"]
fn conformance_lock_helper_process() {
    contract::lock_helper_process();
}
