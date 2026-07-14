//! Lesson 12.1: async functions, joining futures, spawned tasks, and timeouts.

use std::time::Duration;
use tokio::time::{sleep, timeout};

async fn fetch_label(id: u8, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id}")
}

#[tokio::main]
async fn main() {
    let (first, second) = tokio::join!(fetch_label(1, 20), fetch_label(2, 10));
    println!("joined futures: {first}, {second}");

    let owned_name = String::from("spawned");
    let handle = tokio::spawn(async move {
        sleep(Duration::from_millis(5)).await;
        owned_name.to_uppercase()
    });
    println!(
        "task result={}",
        handle.await.expect("task should not panic")
    );

    match timeout(Duration::from_millis(5), fetch_label(3, 50)).await {
        Ok(value) => println!("completed: {value}"),
        Err(_) => println!("item-3 exceeded its timeout"),
    }
}
