//! Binary shell for the idiomatic capstone starter.

use clap::Parser;
use idiomatic_indexer_starter::cli::{Cli, execute};
use std::process::ExitCode;

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
