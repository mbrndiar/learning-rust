use clap::Parser;

use tasks_solution::server::{ServerConfig, run};

#[tokio::main]
async fn main() {
    if let Err(error) = run(ServerConfig::parse()).await {
        eprintln!("tasks-api: {error}");
        std::process::exit(1);
    }
}
