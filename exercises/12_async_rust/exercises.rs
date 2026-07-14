//! Exercises for module 12.

use std::time::Duration;

pub async fn delayed_double(_value: u32, _delay: Duration) -> u32 {
    todo!("await tokio::time::sleep, then double the value")
}

pub async fn double_all(_values: Vec<u32>) -> Result<Vec<u32>, tokio::task::JoinError> {
    todo!("spawn one task per value, await all handles, and sort results")
}

#[tokio::main]
async fn main() {
    println!("Run `cargo test --example ex-12-async` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn doubles_after_waiting() {
        assert_eq!(delayed_double(4, Duration::from_millis(1)).await, 8);
    }

    #[tokio::test]
    async fn waits_for_every_spawned_task() {
        assert_eq!(
            double_all(vec![3, 1, 2]).await.expect("tasks succeed"),
            vec![2, 4, 6]
        );
        assert_eq!(
            double_all(vec![]).await.expect("no tasks"),
            Vec::<u32>::new()
        );
    }
}
