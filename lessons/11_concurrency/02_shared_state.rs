//! Lesson 11.2: `Arc`, `Mutex`, scoped threads, and short lock guards.
//!
//! `Arc<T>` shares ownership across threads by reference counting; `Mutex<T>`
//! guards its data so only one thread touches it at a time. `lock()` returns a
//! guard that releases on drop, so hold it briefly. `thread::scope` lets threads
//! borrow local data because it joins them before the scope returns.

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0_u64));
    let mut handles = Vec::new();

    for _ in 0..4 {
        // `Arc::clone` bumps the reference count; it does not copy the counter.
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1_000 {
                // `lock` returns a guard; `expect` handles a poisoned mutex (a
                // previous holder panicked while holding the lock). The guard
                // drops at the end of each iteration, keeping the hold short.
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
    // `thread::scope` joins its threads before returning, so they may safely
    // borrow `values`. `split_at_mut` hands out two non-overlapping mutable
    // slices, letting each thread own a disjoint half.
    thread::scope(|scope| {
        let (left, right) = values.split_at_mut(3);
        scope.spawn(|| left.iter_mut().for_each(|value| *value *= 2));
        scope.spawn(|| right.iter_mut().for_each(|value| *value *= 3));
    });
    println!("partitioned ownership={values:?}");
}
