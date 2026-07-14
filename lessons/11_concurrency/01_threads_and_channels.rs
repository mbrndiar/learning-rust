//! Lesson 11.1: threads, handles, channels, and ownership transfer.

use std::sync::mpsc;
use std::thread;

fn main() {
    let input = vec![1_u64, 2, 3, 4];
    let worker = thread::spawn(move || input.into_iter().map(|value| value * value).sum::<u64>());
    let total = worker.join().expect("worker should not panic");
    println!("sum of squares={total}");

    let (sender, receiver) = mpsc::channel();
    let mut handles = Vec::new();

    for worker_id in 0..3 {
        let sender = sender.clone();
        handles.push(thread::spawn(move || {
            for item in 0..2 {
                sender
                    .send(format!("worker {worker_id}: item {item}"))
                    .expect("receiver should remain alive");
            }
        }));
    }
    drop(sender);

    let mut messages: Vec<_> = receiver.into_iter().collect();
    messages.sort();
    for message in messages {
        println!("{message}");
    }

    for handle in handles {
        handle.join().expect("producer should not panic");
    }
}
