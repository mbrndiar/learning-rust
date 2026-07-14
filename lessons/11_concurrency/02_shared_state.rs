//! Lesson 11.2: `Arc`, `Mutex`, scoped threads, and short lock guards.

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0_u64));
    let mut handles = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1_000 {
                let mut value = counter.lock().expect("counter invariant should hold");
                *value += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().expect("counter worker should not panic");
    }
    println!(
        "shared counter={}",
        *counter.lock().expect("counter invariant should hold")
    );

    let mut values = [1, 2, 3, 4, 5, 6];
    thread::scope(|scope| {
        let (left, right) = values.split_at_mut(3);
        scope.spawn(|| left.iter_mut().for_each(|value| *value *= 2));
        scope.spawn(|| right.iter_mut().for_each(|value| *value *= 3));
    });
    println!("partitioned ownership={values:?}");
}
