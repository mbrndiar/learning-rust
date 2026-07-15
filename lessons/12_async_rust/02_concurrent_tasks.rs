//! Lesson 12.2: dynamic tasks with bounded concurrency.

use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;

async fn process(id: u8) -> (u8, u16) {
    sleep(Duration::from_millis(u64::from(6 - id) * 3)).await;
    (id, u16::from(id).pow(2))
}

#[tokio::main]
async fn main() {
    const MAX_IN_FLIGHT: usize = 2;
    let mut pending = 1..=5;
    let mut tasks = JoinSet::new();

    for _ in 0..MAX_IN_FLIGHT {
        if let Some(id) = pending.next() {
            tasks.spawn(process(id));
        }
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.expect("task should not panic"));
        // Refill only after one task finishes, so at most MAX_IN_FLIGHT tasks
        // exist at once even when the input is large.
        if let Some(id) = pending.next() {
            tasks.spawn(process(id));
        }
    }
    results.sort_by_key(|(id, _)| *id);

    println!("bounded results={results:?}");
}
