//! Exercises for module 11.

pub fn parallel_sum(_values: Vec<u64>, _worker_count: usize) -> u64 {
    todo!("partition owned values, spawn workers, and join their partial sums")
}

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
