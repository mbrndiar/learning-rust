//! Reference solutions for module 12.

use std::time::Duration;
use tokio::time::sleep;

async fn delayed_double(value: u32, delay: Duration) -> u32 {
    sleep(delay).await;
    value * 2
}

async fn double_all(values: Vec<u32>) -> Result<Vec<u32>, tokio::task::JoinError> {
    let handles: Vec<_> = values
        .into_iter()
        .map(|value| tokio::spawn(delayed_double(value, Duration::from_millis(1))))
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await?);
    }
    results.sort_unstable();
    Ok(results)
}

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    assert_eq!(delayed_double(4, Duration::from_millis(1)).await, 8);
    assert_eq!(double_all(vec![3, 1, 2]).await?, vec![2, 4, 6]);
    println!("Module 12 solutions passed.");
    Ok(())
}
