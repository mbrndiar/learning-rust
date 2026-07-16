use clap::Parser;

use tasks_starter::cli::{Cli, run};

#[tokio::main]
async fn main() {
    if let Err(error) = run(Cli::parse()).await {
        eprintln!("tasks: {error}");
        std::process::exit(1);
    }
}
