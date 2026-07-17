#[tokio::main]
async fn main() {
    let exit = tasks_starter::cli::run_process(std::env::args_os()).await;
    if exit != 0 {
        std::process::exit(exit);
    }
}
