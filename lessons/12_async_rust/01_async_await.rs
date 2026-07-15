//! Lesson 12.1: async functions, joining futures, spawned tasks, and timeouts.
//!
//! An `async fn` returns a future whose work advances only when an executor polls
//! it, usually through `.await` or spawning. `#[tokio::main]` starts the runtime
//! that drives futures to completion. `join!` runs several futures concurrently
//! on one task, `tokio::spawn` schedules independent work, and `timeout` bounds
//! a future.

use std::time::Duration;
use tokio::time::{sleep, timeout};

async fn fetch_label(id: u8, delay_ms: u64) -> String {
    // `.await` yields control until the timer fires, freeing the runtime to make
    // progress on other tasks in the meantime.
    sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id}")
}

#[tokio::main]
async fn main() {
    // `join!` polls both futures concurrently on this single task, so the total
    // wait is about the longer delay (20ms), not the sum of both.
    let (first, second) = tokio::join!(fetch_label(1, 20), fetch_label(2, 10));
    println!("joined futures: {first}, {second}");

    let owned_name = String::from("spawned");
    // `spawn` moves the future onto the runtime to run independently; awaiting
    // the handle yields its result (or a `JoinError` if the task panicked).
    let handle = tokio::spawn(async move {
        sleep(Duration::from_millis(5)).await;
        owned_name.to_uppercase()
    });
    println!(
        "task result={}",
        handle.await.expect("task should not panic")
    );

    // `timeout` returns `Err` if the future is still pending at the deadline,
    // dropping (cancelling) it.
    match timeout(Duration::from_millis(5), fetch_label(3, 50)).await {
        Ok(value) => println!("completed: {value}"),
        Err(_) => println!("item-3 exceeded its timeout"),
    }
}
