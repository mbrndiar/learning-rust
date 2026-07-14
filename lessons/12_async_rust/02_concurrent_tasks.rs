//! Lesson 12.2: dynamic tasks with bounded concurrency.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::sleep;

async fn process(id: u8, permits: Arc<Semaphore>) -> (u8, u16) {
    let _permit = permits
        .acquire_owned()
        .await
        .expect("semaphore should remain open");
    sleep(Duration::from_millis(u64::from(6 - id) * 3)).await;
    (id, u16::from(id).pow(2))
}

#[tokio::main]
async fn main() {
    let permits = Arc::new(Semaphore::new(2));
    let mut tasks = JoinSet::new();

    for id in 1..=5 {
        tasks.spawn(process(id, Arc::clone(&permits)));
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.expect("task should not panic"));
    }
    results.sort_by_key(|(id, _)| *id);

    println!("bounded results={results:?}");
}
