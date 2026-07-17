//! Binary shell for the comparative capstone starter.
//!
//! The process boundary stays thin: parse arguments with clap, hand the parsed
//! [`Cli`] to [`run`], print its payload to stdout on success, or the error to stderr
//! with a failure status. Envelope rendering and exit-code mapping are added as the
//! milestones are implemented.

use clap::Parser;
use comparative_kv_starter::cli::{Cli, run};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(Cli::parse()) {
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
