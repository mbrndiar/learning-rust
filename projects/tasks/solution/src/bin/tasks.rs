//! Binary entry point for the `tasks` CLI client.
//!
//! Forwards process arguments to the library CLI and propagates its exit code,
//! so the stable exit-code policy lives in one place (`cli`).

#[tokio::main]
async fn main() {
    let exit = tasks_solution::cli::run_process(std::env::args_os()).await;
    if exit != 0 {
        std::process::exit(exit);
    }
}
