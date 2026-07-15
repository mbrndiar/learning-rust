//! Lesson 12.2: dynamic tasks with bounded concurrency.
//!
//! A `JoinSet` owns a changing group of spawned tasks and yields their results
//! as they finish. Seeding it with `MAX_IN_FLIGHT` tasks and spawning one more
//! only after each completion caps how many run at once, which is the standard
//! pattern for throttling work across a large input.

use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;

async fn process(id: u8) -> (u8, u16) {
    // Lower ids sleep longer, so results arrive out of input order on purpose.
    sleep(Duration::from_millis(u64::from(6 - id) * 3)).await;
    (id, u16::from(id).pow(2))
}

#[tokio::main]
async fn main() {
    const MAX_IN_FLIGHT: usize = 2;
    let mut pending = 1..=5;
    let mut tasks = JoinSet::new();

    // Prime the set so exactly MAX_IN_FLIGHT tasks are running at the start.
    for _ in 0..MAX_IN_FLIGHT {
        if let Some(id) = pending.next() {
            tasks.spawn(process(id));
        }
    }

    let mut results = Vec::new();
    // `join_next` yields tasks in completion order, not spawn order.
    while let Some(result) = tasks.join_next().await {
        results.push(result.expect("task should not panic"));
        // Refill only after one task finishes, so at most MAX_IN_FLIGHT tasks
        // exist at once even when the input is large.
        if let Some(id) = pending.next() {
            tasks.spawn(process(id));
        }
    }
    // Restore input order for a deterministic printout.
    results.sort_by_key(|(id, _)| *id);

    println!("bounded results={results:?}");
}
