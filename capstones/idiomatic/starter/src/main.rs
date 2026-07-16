//! Binary entry point for the guided idiomatic capstone starter.

use clap::Parser;
use idiomatic_indexer_starter::cli::{Cli, execute};
use serde_json::json;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json_errors = cli.json_errors;
    match execute(cli) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            if json_errors {
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
