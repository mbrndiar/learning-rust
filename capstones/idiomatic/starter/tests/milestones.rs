use idiomatic_indexer_starter as subject;
use std::path::Path;

#[path = "../../tests/contracts/milestones.rs"]
mod contract;

fn program() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_idiomatic-indexer-starter"))
}

#[test]
#[ignore = "Milestone 1 TODO: implement validated domain and tokenization"]
fn milestone_1_validated_domain() {
    contract::milestone_1_validated_domain();
}

#[test]
#[ignore = "Milestone 2 TODO: implement traversal, issues, and CLI boundaries"]
fn milestone_2_traversal_and_issues() {
    contract::milestone_2_traversal_and_issues();
}

#[test]
#[ignore = "Milestone 3 TODO: implement validated atomic persistence"]
fn milestone_3_versioned_storage() {
    contract::milestone_3_versioned_storage();
}

#[test]
#[ignore = "Milestone 4 TODO: implement bounded workers and cancellation"]
fn milestone_4_bounded_concurrency() {
    contract::milestone_4_bounded_concurrency();
}

#[test]
#[ignore = "Milestone 5 TODO: complete subprocess reports and quality gates"]
fn milestone_5_full_cli() {
    contract::milestone_5_full_cli(program());
}
