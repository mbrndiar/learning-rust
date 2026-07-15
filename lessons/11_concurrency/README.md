# 🧵⚡ Module 11: Concurrency

Rust prevents data races by combining ownership with `Send` and `Sync`. Threads
can communicate by transferring messages or by sharing synchronized state.

## 🎯 Learning objectives

After this module, you should be able to spawn and join threads, move owned data
into workers, send values through channels, share state with `Arc<Mutex<T>>`,
limit lock scope, and explain why race freedom does not prevent every
concurrency bug.

## 📨 Message passing

`std::sync::mpsc` channels have one receiver and one or more senders. Sending
usually transfers ownership, so a producer cannot mutate a value after another
thread receives it. Dropping every sender closes the channel and ends receiver
iteration.

Channels make ownership of each message clear, but protocols may still deadlock
or wait forever if senders and receivers disagree about completion.

## 🔒 Shared state

`Arc<T>` provides thread-safe shared ownership. `Mutex<T>` permits one thread at
a time to access its inner value. `Arc<Mutex<T>>` is common, but not a default:
message passing or partitioned ownership may express the design more clearly.

Keep guards short. Do not hold a lock during slow I/O or while acquiring an
unrelated lock. A panic while holding a standard mutex poisons it; callers must
decide whether the protected invariant is still trustworthy.

`Send` means ownership may cross a thread boundary. `Sync` means shared
references may be used across threads. The compiler derives these auto traits
when a type's fields permit them.

## 📘 Lessons

- `01_threads_and_channels.rs` — `move` workers, handles, multiple producers,
  ownership transfer
- `02_shared_state.rs` — `Arc`, `Mutex`, scoped threads, lock boundaries

## 🚀 Running

```bash
cargo run --example lesson-11-threads-channels
cargo run --example lesson-11-shared-state
```

Then practice with
[`exercises/11_concurrency/`](../../exercises/11_concurrency/README.md).

## 🚧 Common mistakes

- Detaching thread handles and never observing panics.
- Keeping an extra sender alive so receiver iteration never finishes.
- Sharing mutable state when ownership could be partitioned.
- Holding a mutex while performing blocking work.
- Acquiring several locks in inconsistent orders.
- Assuming race freedom also prevents deadlock, starvation, or logic errors.

## 🧠 Review questions

1. Why does a spawned closure often need `move`?
2. How does dropping senders signal completion?
3. What separate roles do `Arc` and `Mutex` play?
4. What do `Send` and `Sync` guarantee?
5. Which concurrency failures remain possible in safe Rust?
