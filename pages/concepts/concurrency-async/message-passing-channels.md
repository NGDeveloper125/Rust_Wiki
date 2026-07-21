---
title: "Message passing (channels / mpsc)"
area: "Concurrency & Async"
embedded_support: none
groups: ["Concurrency & Async", "Concurrent / Message-Passing", "Writing Concurrent & Parallel Code", "Message Passing"]
related_syntax: [move]
see_also: ["Threads (std::thread)", "Shared-state concurrency (Mutex, RwLock)", "Send & Sync", "Move semantics"]
---

## Explanation

A channel is a queue-like pipe connecting two or more threads: one side
sends values into it, the other side receives them, and the standard
library's `std::sync::mpsc` module (multiple-producer, single-consumer)
provides the classic form — any number of cloned `Sender` handles can feed
values in, but only one `Receiver` drains them, in the order they were
sent. Sending a value moves it through the channel: ownership transfers
from the sending thread to whichever thread eventually receives it, so
there is no moment where both threads hold the data at once (see
[Move semantics](../ownership-borrowing/move-semantics.md)).

This is the concurrency model summed up by the slogan "don't communicate
by sharing memory; share memory by communicating" — instead of multiple
threads reaching into the same location and coordinating through locks
(see [Shared-state concurrency](shared-state-concurrency.md)), each piece
of data has a single owner at any moment and moves between threads through
an explicit handoff. This sidesteps an entire category of concurrency
bugs: there's no lock to forget, no critical section to keep short, and no
way for two threads to observe the data mid-mutation, because the data is
simply never in two places' hands simultaneously.

Channels are a natural fit for pipeline- and worker-pool-shaped programs:
a producer thread generates work items and sends them, a pool of consumer
threads receives and processes them, and results can flow back through a
second channel. Because `Sender` and `Receiver` are ordinary values, they
compose with everything else in the ownership system — cloning a `Sender`
for each producer thread, moving a `Receiver` into the one consuming
thread, and letting `Drop` close the channel automatically once every
`Sender` has gone out of scope (the receive loop then ends naturally
instead of needing a manual shutdown signal).

Message passing and shared-state concurrency aren't mutually exclusive —
real programs often use both, picking whichever fits a given piece of
data. A value that's naturally produced by one thread and consumed by
another (a job queue, a stream of sensor readings, a pipeline stage) suits
a channel; a value that many threads need to read *and* update
concurrently (a shared counter, a connection pool) suits a
[Mutex or RwLock](shared-state-concurrency.md) instead.

## Basic usage example

```
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel(); // <- tx sends into the channel, rx receives from it

thread::spawn(move || {
    tx.send("done").unwrap(); // <- ownership of the value moves into the channel
});

println!("{}", rx.recv().unwrap()); // blocks until a value arrives
```

## Best practices & deeper information

### Scenario: Message passing between threads

An order-processing service reads incoming orders off one channel and
fans results back out through another, keeping the intake and processing
stages fully decoupled.

```
use std::sync::mpsc;
use std::thread;

struct Order { id: u64, total_cents: u64 }

let (order_tx, order_rx) = mpsc::channel::<Order>();
let (result_tx, result_rx) = mpsc::channel::<u64>();

thread::spawn(move || {
    for order in order_rx { // <- receives each Order as ownership is handed across
        result_tx.send(order.id).unwrap(); // <- forwards a result through a second channel
    }
});

order_tx.send(Order { id: 1, total_cents: 2500 }).unwrap();
drop(order_tx); // closes the channel so the receiving loop above can end

for id in result_rx {
    println!("processed order {id}");
}
```

**Why this way:** chaining two single-purpose channels keeps the intake
and processing stages independently testable and replaceable, matching the
[Rust Book's](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
framing of channels as the default choice for handing work between threads
before reaching for shared state.

### Scenario: Multi-threading

A worker pool distributes incoming jobs to a fixed number of threads by
cloning the `Sender` once per worker, while a single shared `Receiver`
(wrapped for multi-consumer access) hands each job to whichever worker
asks for it next.

```
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

let (tx, rx) = mpsc::channel::<u32>();
let rx = Arc::new(Mutex::new(rx)); // <- one Receiver, shared so multiple worker threads can pull from it

let mut workers = Vec::new();
for worker_id in 0..4 {
    let rx = Arc::clone(&rx);
    workers.push(thread::spawn(move || {
        while let Ok(job) = rx.lock().unwrap().recv() { // <- lock only long enough to pull one job
            println!("worker {worker_id} handling job {job}");
        }
    }));
}

for job in 0..8 {
    tx.send(job).unwrap();
}
drop(tx);

for worker in workers {
    worker.join().unwrap();
}
```

**Why this way:** `mpsc::Receiver` isn't `Sync`, so sharing the *receiving*
end across several worker threads needs a `Mutex` around it even though the
data flowing through the channel is still handed off by value — the
[`std::sync::mpsc` docs](https://doc.rust-lang.org/std/sync/mpsc/index.html)
describe this single-consumer design, and wrapping the receiver is the
standard pattern for a multi-worker pool built on it.

### Scenario: Sharing state across threads

A background exporter thread needs a way to be told to stop cleanly —
sending a dedicated shutdown message down the same channel used for real
work avoids introducing a second synchronization primitive just for
signaling.

```
use std::sync::mpsc;
use std::thread;

enum Task {
    Export(String),
    Shutdown, // <- a plain message, no separate flag or condition variable needed
}

let (tx, rx) = mpsc::channel();

let exporter = thread::spawn(move || {
    for task in rx {
        match task {
            Task::Export(name) => println!("exporting {name}"),
            Task::Shutdown => break, // <- stops the loop on request
        }
    }
});

tx.send(Task::Export("report.csv".into())).unwrap();
tx.send(Task::Shutdown).unwrap();
exporter.join().unwrap();
```

**Why this way:** modeling shutdown as a message keeps all inter-thread
coordination going through the same channel instead of adding an
`AtomicBool` flag the receiver has to poll separately — the
[Rust Book](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
treats channels as a general-purpose way to coordinate, not just to
transfer data.

## Embedded Rust Notes

**No support.** `std::sync::mpsc` is built on OS-level blocking
(`recv` parks the calling thread until a value or a closed channel wakes
it), so it depends on both `std` and an operating system scheduler,
neither of which bare-metal or RTOS-free embedded targets have. Embedded
code that needs a bounded producer/consumer queue between contexts (a
main loop and an interrupt handler, or two async tasks) typically reaches
for a fixed-capacity `heapless::spsc` queue, or an async channel from an
executor like `embassy` (`embassy_sync::channel`) that suspends a task
instead of blocking an OS thread.
