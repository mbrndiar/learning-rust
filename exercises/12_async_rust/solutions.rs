//! Reference solutions for module 12.

use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;

async fn delayed_double(value: u32, delay: Duration) -> u32 {
    sleep(delay).await;
    value * 2
}

async fn double_all(
    values: Vec<u32>,
    max_in_flight: usize,
) -> Result<Vec<u32>, tokio::task::JoinError> {
    let mut pending = values.into_iter();
    let mut tasks = JoinSet::new();

    for _ in 0..max_in_flight.max(1) {
        if let Some(value) = pending.next() {
            tasks.spawn(delayed_double(value, Duration::from_millis(1)));
        }
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result?);
        if let Some(value) = pending.next() {
            tasks.spawn(delayed_double(value, Duration::from_millis(1)));
        }
    }
    results.sort_unstable();
    Ok(results)
}

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    assert_eq!(delayed_double(4, Duration::from_millis(1)).await, 8);
    assert_eq!(double_all(vec![3, 1, 2], 2).await?, vec![2, 4, 6]);
    println!("Module 12 solutions passed.");
    Ok(())
}
