//! Binary shell for the comparative capstone solution.
//!
//! The process boundary is intentionally tiny: forward the OS arguments (minus
//! `argv[0]`) to [`run_process`], print the rendered stdout envelope, and exit with
//! the spec-defined status. All parsing, validation, and rendering live in the
//! library so the same logic is exercised directly by the conformance tests.

use comparative_kv_solution::cli::run_process;
use std::process::ExitCode;

fn main() -> ExitCode {
    let output = run_process(std::env::args_os().skip(1));
    println!("{}", output.stdout);
    ExitCode::from(output.exit_code)
}
