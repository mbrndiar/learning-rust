//! Binary entry point: parse arguments, run the command, map the outcome to an
//! exit code. All real logic lives in the `task_manager` library so it can be
//! unit- and integration-tested without spawning a process.

use clap::Parser;
use std::process::ExitCode;
use task_manager::cli::{Cli, run};

fn main() -> ExitCode {
    // Success prints to stdout; any error prints to stderr and yields a nonzero
    // exit status so scripts and shells can detect failure.
    match run(Cli::parse()) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
