//! Binary entry point for the idiomatic capstone solution.
//!
//! This is the process boundary: it parses arguments, delegates all real work to
//! [`cli::execute`], and translates the outcome into stdout, stderr, and an exit
//! code. Success prints the command's JSON/text payload on stdout; failures print
//! a diagnostic on stderr and never on stdout, so normal output stays clean.

use clap::Parser;
use idiomatic_indexer_solution::cli::{Cli, execute};
use serde_json::json;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    // Capture the flag before `execute` consumes `cli`; it selects the stderr shape.
    let json_errors = cli.json_errors;
    match execute(cli) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            if json_errors {
                // One structured object; a missing code (only the scaffold path)
                // reports as the write/worker category.
                eprintln!(
                    "{}",
                    json!({
                        "error": {
                            "code": error.code().map_or("worker_failed", |code| code.as_str()),
                            "message": error.to_string(),
                            "details": {}
                        }
                    })
                );
            } else {
                eprintln!("{error}");
            }
            ExitCode::from(error.exit_code())
        }
    }
}
