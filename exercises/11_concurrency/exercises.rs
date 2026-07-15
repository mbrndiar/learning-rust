//! Exercises for module 11: threads and channels.
//!
//! Implement each `todo!()` body, then run the example tests. Do not change any
//! signature.

/// Sum `values` in parallel across up to `worker_count.max(1)` threads.
///
/// Takes ownership of `values` and partitions it among the workers, then joins
/// their partial sums. An empty input returns 0, and a `worker_count` of 0 is
/// treated as a single worker.
pub fn parallel_sum(_values: Vec<u64>, _worker_count: usize) -> u64 {
    todo!("partition owned values, spawn workers, and join their partial sums")
}

/// Collect one `"worker-{i}"` message from each of `workers` threads.
///
/// Each thread sends its message through a channel; the results are gathered and
/// returned sorted, so the output is deterministic regardless of thread timing.
pub fn worker_messages(_workers: usize) -> Vec<String> {
    todo!("send one message per worker through a channel and sort the result")
}

fn main() {
    println!("Run `cargo test --example ex-11-concurrency` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_values_with_reasonable_worker_counts() {
        assert_eq!(parallel_sum(vec![1, 2, 3, 4, 5], 2), 15);
        assert_eq!(parallel_sum(vec![], 4), 0);
        assert_eq!(parallel_sum(vec![7], 0), 7);
    }

    #[test]
    fn receives_every_worker_message() {
        assert_eq!(
            worker_messages(3),
            vec![
                String::from("worker-0"),
                String::from("worker-1"),
                String::from("worker-2"),
            ]
        );
    }
}
