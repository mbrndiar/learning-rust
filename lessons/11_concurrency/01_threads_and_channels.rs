//! Lesson 11.1: threads, handles, channels, and ownership transfer.
//!
//! `thread::spawn` runs a closure on a new OS thread and returns a `JoinHandle`;
//! `join` waits for it and yields the closure's value. `move` transfers owned
//! data into a thread so it can outlive the spawning scope. An `mpsc` channel
//! passes values between threads (many senders, one receiver), moving on `send`.

use std::sync::mpsc;
use std::thread;

fn main() {
    let input = vec![1_u64, 2, 3, 4];
    // `move` transfers ownership of `input` into the thread's closure.
    let worker = thread::spawn(move || input.into_iter().map(|value| value * value).sum::<u64>());
    // `join` blocks until the thread finishes and returns its computed value.
    let total = worker.join().expect("worker should not panic");
    println!("sum of squares={total}");

    let (sender, receiver) = mpsc::channel();
    let mut handles = Vec::new();

    for worker_id in 0..3 {
        // Each thread needs its own sender clone; they all feed the one receiver.
        let sender = sender.clone();
        handles.push(thread::spawn(move || {
            for item in 0..2 {
                sender
                    .send(format!("worker {worker_id}: item {item}"))
                    .expect("receiver should remain alive");
            }
        }));
    }
    // Drop the original sender so the receiver's iterator can end once every
    // cloned sender has also been dropped.
    drop(sender);

    // `into_iter` on the receiver yields messages until all senders are gone.
    let mut messages: Vec<_> = receiver.into_iter().collect();
    messages.sort();
    for message in messages {
        println!("{message}");
    }

    for handle in handles {
        handle.join().expect("producer should not panic");
    }
}
