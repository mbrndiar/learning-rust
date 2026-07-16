//! Binary shell for the comparative capstone solution.

use comparative_kv_solution::cli::run_process;
use std::process::ExitCode;

fn main() -> ExitCode {
    let output = run_process(std::env::args_os().skip(1));
    println!("{}", output.stdout);
    ExitCode::from(output.exit_code)
}
