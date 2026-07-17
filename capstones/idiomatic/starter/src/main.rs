//! Binary entry point for the guided idiomatic capstone starter.
//!
//! This process boundary is already wired: it parses arguments, delegates to
//! [`cli::execute`], and maps the outcome to stdout, stderr, and an exit code.
//! Success prints the payload on stdout; failures print a diagnostic on stderr
//! only. It is complete on purpose so you can focus on the library modules.

use clap::Parser;
use idiomatic_indexer_starter::cli::{Cli, execute};
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
                // One structured object; a missing code (the scaffold path) reports
                // as the write/worker category.
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
