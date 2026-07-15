//! Reference solutions for module 11.

use std::sync::mpsc;
use std::thread;

fn parallel_sum(values: Vec<u64>, worker_count: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }

    // Clamp workers to at least 1 and at most one per element, then size chunks
    // so every worker gets a contiguous, non-overlapping slice.
    let workers = worker_count.max(1).min(values.len());
    let chunk_size = values.len().div_ceil(workers);
    let handles: Vec<_> = values
        .chunks(chunk_size)
        .map(|chunk| {
            // Copy the borrowed chunk into an owned Vec the thread can take.
            let owned = chunk.to_vec();
            thread::spawn(move || owned.into_iter().sum::<u64>())
        })
        .collect();

    handles
        .into_iter()
        .map(|handle| handle.join().expect("worker should not panic"))
        .sum()
}

fn worker_messages(workers: usize) -> Vec<String> {
    let (sender, receiver) = mpsc::channel();
    let handles: Vec<_> = (0..workers)
        .map(|worker| {
            let sender = sender.clone();
            thread::spawn(move || {
                sender
                    .send(format!("worker-{worker}"))
                    .expect("receiver should remain alive");
            })
        })
        .collect();
    // Drop the original sender so the receiver stops once every clone is gone.
    drop(sender);

    let mut messages: Vec<_> = receiver.into_iter().collect();
    for handle in handles {
        handle.join().expect("worker should not panic");
    }
    messages.sort();
    messages
}

fn main() {
    assert_eq!(parallel_sum(vec![1, 2, 3, 4, 5], 2), 15);
    assert_eq!(worker_messages(3), vec!["worker-0", "worker-1", "worker-2"]);
    println!("Module 11 solutions passed.");
}
