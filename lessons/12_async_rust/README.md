# 🌊⚡ Module 12: Async Rust

Async Rust represents suspendable work as futures. An executor polls futures
when they can make progress; `.await` cooperatively yields instead of blocking
an operating-system thread.

## 🎯 Learning objectives

After this module, you should be able to explain what an async function returns,
await futures, run independent work concurrently, spawn owned `'static` tasks,
observe task failures, enforce timeouts, and bound fan-out with a semaphore.

## 🔮 Futures and `.await`

Calling an `async fn` creates a future; it does not run the body to completion.
An executor such as [Tokio](https://tokio.rs/) repeatedly polls the future.
`.await` suspends the current async task until the awaited future is ready.

Do not call blocking filesystem, network, sleep, or CPU-heavy code directly
inside an async task. Use async-aware APIs or deliberately move blocking work to
`spawn_blocking`. Async improves waiting-heavy concurrency; it does not make CPU
work intrinsically faster.

## 🧭 Concurrency structure

- `tokio::join!` drives a known set of futures concurrently in one task.
- `tokio::spawn` schedules an independently owned task and returns a
  `JoinHandle`; await it so panics and cancellation are observed.
- `JoinSet` owns a dynamic group and yields completions.
- `timeout` bounds how long a future may take.
- `Semaphore` limits concurrent access or fan-out.

Cancellation happens when a future is dropped. Code that owns external
resources must be cancellation-safe or perform explicit cleanup.

## 📘 Lessons

- `01_async_await.rs` — async functions, `join!`, task spawning, timeout
- `02_concurrent_tasks.rs` — `JoinSet`, bounded concurrency, deterministic
  aggregation

## 🚀 Running

```bash
cargo run --example lesson-12-async-await
cargo run --example lesson-12-concurrent-tasks
```

Then practice with [`exercises/12_async_rust/`](../../exercises/12_async_rust/README.md)
and continue to the [capstone project](../../project/task_manager/README.md).

## 🚧 Common mistakes

- Creating a future and never awaiting or spawning it.
- Using `std::thread::sleep` inside an async task.
- Spawning tasks and ignoring their `JoinHandle`.
- Creating unbounded tasks for untrusted or very large input.
- Holding a synchronous mutex guard across `.await`.
- Assuming async means parallel CPU execution.

## 🧠 Review questions

1. What happens when an `async fn` is called?
2. How does `.await` differ from blocking a thread?
3. When should work use `join!` versus `spawn`?
4. Why must spawned tasks usually own `'static` data?
5. How do timeouts and semaphores protect an application?
