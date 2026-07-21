---
title: "Threads (std::thread)"
area: "Concurrency & Async"
embedded_support: none
groups: ["Concurrency & Async", "Writing Concurrent & Parallel Code", "Multithreading", "Message Passing"]
related_syntax: [move]
see_also: ["Message passing (channels / mpsc)", "Shared-state concurrency (Mutex, RwLock)", "Send & Sync", "Shared ownership (Rc & Arc)"]
---

## Explanation

A thread is an independent, OS-scheduled sequence of execution within a
program — `std::thread::spawn` hands a closure to the operating system,
which runs it in parallel with everything else the program is doing,
potentially on a different CPU core. This is Rust's most direct route to
parallelism: unlike async tasks, which cooperatively share a small number
of underlying threads, an `std::thread` is a genuine OS-level unit of
execution with its own stack, scheduled preemptively by the kernel.

Threads exist for two distinct reasons that are easy to conflate: using
multiple CPU cores at once for CPU-bound work (splitting a computation
across worker threads), and letting one long-running or blocking operation
proceed without freezing the rest of the program. Rust makes no attempt to
hide threads behind an implicit runtime — spawning one is an explicit,
visible decision, and the type system tracks the consequences: a spawned
closure must be `'static` and its captures must be [Send](send-and-sync.md),
because the new thread might outlive the scope that created it and nothing
guarantees the two threads won't touch the same data concurrently.

Rust's headline claim of "fearless concurrency" comes directly from this
group of concepts: the borrow checker and the `Send`/`Sync` marker traits
turn most data-race bugs — the kind that plague threaded code in other
systems languages — into compile errors rather than runtime heisenbugs.
A thread that needs to share data with its spawner reaches for either
[message passing](message-passing-channels.md) (send the data, don't share
it) or [shared-state concurrency](shared-state-concurrency.md) (share it,
but only through a lock) — `std::thread` itself is deliberately minimal,
providing just the primitive of "run this closure on another thread" and
leaving the data-sharing strategy to those sibling concepts.

`thread::spawn` requires `'static` data because the spawned thread's
lifetime is independent of the caller's — the standard workaround is
[Arc](../ownership-borrowing/shared-ownership-rc-arc.md) for genuinely
shared data, or `thread::scope` (stable since Rust 1.63) for the common
case where the spawning function is willing to block until every child
thread finishes, which lets threads safely borrow local, non-`'static`
data instead.

## Basic usage example

```
use std::thread;

let handle = thread::spawn(|| { // <- spawns a new OS thread running this closure
    println!("hello from a spawned thread");
});

handle.join().unwrap(); // blocks until the spawned thread finishes
```

## Best practices & deeper information

### Scenario: Multi-threading

A batch job that resizes a folder of images can split the work across
several worker threads, joining them all before reporting completion —
`thread::spawn` for the parallel work, `thread::scope` so the workers can
borrow the shared file list without needing to own or clone it.

```
use std::thread;

fn resize_all(paths: &[String]) {
    thread::scope(|scope| { // <- scoped threads may borrow `paths` because scope() blocks until they finish
        for chunk in paths.chunks(paths.len().div_ceil(4).max(1)) {
            scope.spawn(move || { // <- each worker thread processes its slice in parallel
                for path in chunk {
                    println!("resizing {path}");
                }
            });
        }
    }); // <- all worker threads are joined automatically here
}

resize_all(&["a.png".to_string(), "b.png".to_string(), "c.png".to_string()]);
```

**Why this way:** `thread::scope` lets threads borrow stack data instead of
requiring `'static` + `Arc`, because the compiler can guarantee every
spawned thread is joined before the scope ends — the
[Rust Book](https://doc.rust-lang.org/book/ch21-01-single-threaded.html)
and the [`std::thread::scope` docs](https://doc.rust-lang.org/std/thread/fn.scope.html)
both recommend scoped threads whenever the parent can afford to wait.

### Scenario: Sharing state across threads

A worker pool that needs to report progress back into a shared counter
must move an owned handle into each spawned closure — `thread::spawn`'s
`'static` requirement means the counter can't just be borrowed, it has to
be an owned, cloneable, thread-safe handle.

```
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::thread;

let jobs_completed = Arc::new(AtomicU64::new(0));
let mut handles = Vec::new();

for worker_id in 0..4 {
    let jobs_completed = Arc::clone(&jobs_completed); // <- owned handle moved into the thread, not borrowed
    handles.push(thread::spawn(move || {
        // ... process this worker's share of jobs ...
        jobs_completed.fetch_add(1, Ordering::Relaxed);
        println!("worker {worker_id} done");
    }));
}

for handle in handles {
    handle.join().unwrap();
}
```

**Why this way:** `thread::spawn` requires `'static` captures because a
spawned thread's lifetime isn't tied to its caller's stack frame, so
sharing data across an unscoped spawn boundary means giving each thread its
own owned, reference-counted handle rather than a borrow — the
[Rust Book's shared-state chapter](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
covers this `Arc`-per-thread pattern directly.

### Scenario: Message passing between threads

A producer thread reading sensor input can hand each reading off to a
dedicated processing thread through a channel, rather than both threads
touching a shared buffer directly.

```
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || { // <- producer thread owns tx, sends readings as they arrive
    for reading in [21.4, 21.6, 22.0] {
        tx.send(reading).unwrap();
    }
});

for reading in rx { // main thread receives each reading in order
    println!("sensor reading: {reading}");
}
```

**Why this way:** letting the channel carry ownership of each message means
the producer and consumer threads never need a shared lock at all — the
[Rust Book](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
presents channels as the preferred first choice for inter-thread
communication before reaching for shared state.

## Embedded Rust Notes

**No support.** `std::thread::spawn` needs an operating system to create
and preemptively schedule threads, which most embedded targets simply
don't have — bare-metal firmware runs on one hardware thread of execution
(plus interrupt handlers), and `#![no_std]` doesn't provide the `thread`
module at all. Devices running a real RTOS (FreeRTOS, Zephyr, RTIC) get an
equivalent notion of concurrent tasks from that RTOS's own primitives
instead, not from `std::thread`; async executors like `embassy` are the
usual no_std answer to running several logical tasks concurrently on a
single core.
