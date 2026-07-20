---
title: "Shared-state concurrency (Mutex, RwLock)"
area: "Concurrency & Async"
embedded_support: none
groups: ["Concurrency & Async", "Concurrent / Message-Passing", "Writing Concurrent & Parallel Code", "Sharing & Mutating Data Safely", "Multithreading"]
related_syntax: []
see_also: ["Threads (std::thread)", "Message passing (channels / mpsc)", "Send & Sync", "Interior mutability (Cell & RefCell)", "Shared ownership (Rc & Arc)"]
---

## Explanation

Shared-state concurrency lets multiple threads touch the same piece of
data by guarding it with a lock rather than routing every access through a
message. `std::sync::Mutex<T>` grants exclusive access to its contents one
thread at a time — calling `.lock()` blocks until no other thread holds
the lock, then hands back a guard through which the data can be read or
mutated; `RwLock<T>` refines this by allowing any number of concurrent
readers, or exactly one writer, which pays off when reads vastly outnumber
writes.

This is the counterpart to [message passing](message-passing-channels.md):
instead of moving ownership of a value between threads, a `Mutex`/`RwLock`
lets several threads share a single instance of it directly, at the cost
of needing to coordinate who's touching it and when. Both are legitimate
answers to "multiple threads, one piece of data" — channels suit data that
flows from a producer to a consumer, while a lock suits data that many
threads genuinely need to read and update in place, like a shared cache or
connection pool. Both are frequently reached for together in the same
program, on different pieces of state.

Rust's `Mutex<T>` differs from locks in most other languages in one
important way: the lock and the data it protects are the same value,
rather than a bare lock guarding data by convention. There is no way to
touch the `T` inside a `Mutex<T>` without going through `.lock()` first,
so the type system — not documentation or discipline — guarantees every
access is synchronized. This is the same idea as
[interior mutability](../ownership-borrowing/interior-mutability.md)
(`Cell`/`RefCell`), extended to be safe across threads: `RefCell` catches
misuse at runtime with a single-threaded borrow check, while `Mutex`
enforces mutual exclusion across threads via blocking.

Because a `Mutex`/`RwLock` needs to be reachable from every thread that
locks it, it's almost always found wrapped in an
[`Arc`](../ownership-borrowing/shared-ownership-rc-arc.md) — `Arc<Mutex<T>>`
is common enough to be a recognizable idiom on its own: `Arc` provides the
multi-owner handle, `Mutex` provides the safe mutation through it. The
guard returned by `.lock()` also owns the unlocking: it releases the lock
automatically via `Drop` when it goes out of scope, so keeping the guard's
scope as short as possible — rather than holding it across unrelated work
— is the main discipline shared-state code has to maintain by hand.

## Basic usage example

```
use std::sync::Mutex;

let counter = Mutex::new(0);

{
    let mut guard = counter.lock().unwrap(); // <- blocks until the lock is free, returns a guard
    *guard += 1;
} // guard drops here, releasing the lock

println!("{}", *counter.lock().unwrap());
```

## Best practices & deeper information

### Scenario: Sharing state across threads

A metrics collector needs several worker threads to increment shared
counters concurrently — wrapping the counters in `Arc<Mutex<T>>` lets
every thread hold its own cheap handle to the same underlying data.

```
use std::sync::{Arc, Mutex};
use std::thread;

struct Metrics { requests_handled: u64, errors: u64 }

let metrics = Arc::new(Mutex::new(Metrics { requests_handled: 0, errors: 0 }));
let mut handles = Vec::new();

for _ in 0..4 {
    let metrics = Arc::clone(&metrics);
    handles.push(thread::spawn(move || {
        let mut m = metrics.lock().unwrap(); // <- exclusive access to Metrics for this scope only
        m.requests_handled += 1;
    })); // <- guard drops at end of closure, releasing the lock immediately
}

for handle in handles {
    handle.join().unwrap();
}

println!("{}", metrics.lock().unwrap().requests_handled);
```

**Why this way:** keeping the locked scope limited to just the field
update, rather than holding the guard across the rest of the closure,
minimizes contention between the four worker threads — the
[Rust Book](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
covers `Arc<Mutex<T>>` as the standard combination for sharing mutable
state across threads.

### Scenario: Multi-threading

A read-heavy in-memory config cache, refreshed occasionally but read
constantly by request-handling threads, is a better fit for `RwLock` than
`Mutex` — concurrent readers don't need to block each other.

```
use std::sync::{Arc, RwLock};
use std::thread;

struct Config { max_connections: u32 }

let config = Arc::new(RwLock::new(Config { max_connections: 100 }));

let readers: Vec<_> = (0..3).map(|_| {
    let config = Arc::clone(&config);
    thread::spawn(move || {
        let cfg = config.read().unwrap(); // <- multiple readers may hold this concurrently
        println!("max_connections = {}", cfg.max_connections);
    })
}).collect();

{
    let mut cfg = config.write().unwrap(); // <- exclusive: waits for all readers to finish first
    cfg.max_connections = 200;
}

for reader in readers {
    reader.join().unwrap();
}
```

**Why this way:** `RwLock` lets read-only threads proceed in parallel
instead of serializing on a single exclusive lock, which pays off exactly
when reads dominate writes — the
[`std::sync::RwLock` docs](https://doc.rust-lang.org/std/sync/struct.RwLock.html)
recommend it for this read-heavy access pattern over a plain `Mutex`.

### Scenario: Designing a public API

A type that wraps shared, mutable state internally should keep the lock
as a private implementation detail, exposing only methods that lock,
mutate, and unlock in one step — callers should never see a raw guard.

```
use std::sync::{Arc, Mutex};

pub struct JobQueue {
    jobs: Mutex<Vec<String>>, // <- private: the lock is an implementation detail, not part of the API
}

impl JobQueue {
    pub fn new() -> Arc<Self> {
        Arc::new(JobQueue { jobs: Mutex::new(Vec::new()) })
    }

    pub fn push(&self, job: String) { // <- locks, mutates, unlocks — caller never touches the Mutex directly
        self.jobs.lock().unwrap().push(job);
    }

    pub fn pop(&self) -> Option<String> {
        self.jobs.lock().unwrap().pop()
    }
}
```

**Why this way:** hiding the `Mutex` behind methods that lock only for the
duration of one operation prevents callers from accidentally holding a
guard too long or forgetting to lock at all — a narrow, well-defined API
surface is exactly what the
[API Guidelines](https://rust-lang.github.io/api-guidelines/) recommend
for any type with an invariant (here, "the lock is always released
promptly") that callers shouldn't be able to violate.

## Embedded Rust Notes

**No support.** `std::sync::Mutex` and `RwLock` are built on OS blocking
primitives (futexes or their platform equivalent) to park a thread when
the lock is contended, which requires both `std` and an OS scheduler that
bare-metal and many RTOS-free embedded targets don't have. Embedded code
that needs to share mutable state safely between a main loop and an
interrupt handler typically reaches for a `critical-section`-based mutex
instead — `cortex_m::interrupt::Mutex<RefCell<T>>` on Cortex-M, or the
portable `critical-section` crate — which achieves exclusion by briefly
disabling interrupts rather than blocking a thread that doesn't exist.
