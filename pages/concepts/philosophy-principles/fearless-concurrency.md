---
title: "Fearless concurrency"
area: "Rust Philosophy & Design Principles"
embedded_support: full
groups: ["Rust Philosophy & Design Principles", "Concurrent / Message-Passing", "Writing Concurrent & Parallel Code", "Unique to Rust", "Multithreading"]
related_syntax: []
see_also: ["Send & Sync", "Threads (std::thread)", "Shared-state concurrency (Mutex, RwLock)", "Message passing (channels / mpsc)", "The borrow checker", "Zero-cost abstractions"]
---

## Explanation

"Fearless concurrency" is the Rust team's own name for a specific promise:
writing multi-threaded code shouldn't require the constant, low-grade
vigilance that concurrent programming demands in C or C++, where sharing
mutable data across threads compiles cleanly whether it's safe or not, and
a mistake might not surface until a customer hits it under production load
months later. Rust backs this promise with the same mechanism it uses for
memory safety generally: ownership and borrowing, checked entirely at
compile time, extended across the thread boundary by two marker traits,
[`Send` and `Sync`](../concurrency-async/send-and-sync.md).

The mechanics are a direct extension of single-threaded borrowing. A data
race requires two things: more than one thread touching the same memory,
with at least one of them writing, and no synchronization between them.
Rust's borrow checker already guarantees that a value has either any
number of live shared references or exactly one mutable reference, never
both at once — see [the borrow checker](../ownership-borrowing/borrow-checker.md)
and [mutable borrowing](../ownership-borrowing/mutable-borrowing.md) for
the single-threaded version of this rule. `Send` and `Sync` simply state
whether that guarantee still holds once a value crosses into another
thread: `Send` governs whether ownership can move to another thread at
all, `Sync` governs whether `&T` can safely be held by more than one
thread at once. Because `std::thread::spawn` requires its closure's
captures to satisfy these bounds, code that would share genuinely
thread-unsafe data — an `Rc<T>`'s non-atomic reference count, for
instance — simply fails to compile, rather than shipping a race condition.
[Threads](../concurrency-async/threads.md),
[shared-state locking](../concurrency-async/shared-state-concurrency.md),
and [message-passing channels](../concurrency-async/message-passing-channels.md)
are the three concrete places this guarantee actually does its work.

It's important to be precise about what "fearless" covers, because it's
narrower than "concurrency bugs are impossible." The compiler eliminates
*data races* specifically — simultaneous, unsynchronized access with at
least one write — because that class of bug is exactly what ownership and
`Send`/`Sync` are built to reject. It does not, and cannot, eliminate
deadlocks (two threads each waiting on a lock the other already holds),
logic races (two threads correctly synchronized individually but combined
in the wrong order to produce a wrong result), or plain algorithmic bugs.
The type system checks that access to shared data is synchronized; it has
no opinion on whether the synchronization you wrote is logically correct.
Even the failure modes Rust does surface are handled as visible, typed
signals rather than silent corruption — a panic while holding a `Mutex`
poisons the lock rather than leaving the guarded data in a
partially-updated state nobody notices.

Async code carries a version of the same story into a different
execution model: `async`/`await` (see [async/await](../concurrency-async/async-await.md)
and [futures](../concurrency-async/futures.md)) still checks `Send` bounds
on anything a spawned task captures across an `.await` point, so the same
"won't compile if it isn't safe to hand to the executor" guarantee applies
there too — the risk profile just shifts from data races toward
accidentally blocking an executor thread, which the type system does not
catch and which is why blocking calls inside async code are treated as a
correctness bug in their own right rather than merely a performance one.

Taken together, this is why Rust gets reached for in domains that used to
be considered too risky to parallelize casually — browser engine
components, database internals, high-throughput network services. The
promise was never "concurrent code is easy to design correctly." It's
narrower and more honest than that: an entire, historically brutal-to-debug
category of bug is caught before the program ever runs, by the same
compiler that was already checking ownership for the single-threaded case.

## Basic usage example

```
use std::thread;

let total = 0;

// thread::spawn(|| { total += 1; }); // would fail to compile: this closure would need to outlive `total`,
                                       // which it can't safely borrow across a `'static`-bound thread

let handle = thread::spawn(move || { // <- ownership of `total` moves into the thread instead of being borrowed
    println!("received {total}");
});

handle.join().unwrap();
```

Fixing the commented-out version means making the choice explicit: move
`total`'s ownership into the thread (as the working version above does),
or wrap it in something the compiler already knows is safe to share, like
an atomic or a `Mutex`. Either way, the compiler forces that decision
before two threads can quietly race.

## Best practices & deeper information

### Scenario: Multi-threading

A pending-orders queue touched by more than one worker thread has to be
reshaped before it compiles at all — sharing a bare `&mut` across threads
is rejected outright, which is exactly the guarantee "fearless" refers to.

```
use std::sync::{Arc, Mutex};
use std::thread;

struct OrderQueue { pending: Vec<u32> }

let queue = OrderQueue { pending: vec![101, 102, 103] };

// AVOID: sharing `&mut queue` across threads — rejected at compile time
// thread::spawn(|| queue.pending.push(104));
// thread::spawn(|| queue.pending.pop());
// error: closure may outlive the current function, but it borrows `queue`, which is owned by the current function

// PREFER: make the shared, synchronized access explicit
let queue = Arc::new(Mutex::new(queue));
let producer = Arc::clone(&queue);
let consumer = Arc::clone(&queue);

thread::spawn(move || producer.lock().unwrap().pending.push(104)); // <- Send/Sync + Mutex checked at compile time
thread::spawn(move || { consumer.lock().unwrap().pending.pop(); });
```

**Why this way:** the compiler rejects the shared `&mut` before the
program can run at all, rather than letting two threads race on the same
`Vec` — the
[Rust Book's shared-state chapter](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
covers `Arc<Mutex<T>>` as the standard fix once data genuinely needs to be
touched from more than one thread.

### Scenario: Sharing state across threads

A shared job counter needs to survive one worker panicking mid-update
without silently corrupting the count — Rust's `Mutex` shows fearless
concurrency's honest edge: it doesn't prevent the panic, but it turns what
would otherwise be silent data corruption into a visible, typed error.

```
use std::sync::{Arc, Mutex};
use std::thread;

let jobs_completed = Arc::new(Mutex::new(0u32));

let worker = Arc::clone(&jobs_completed);
let handle = thread::spawn(move || {
    let mut count = worker.lock().unwrap();
    *count += 1;
    panic!("worker crashed mid-update"); // <- lock is still held by this guard when the panic unwinds
});
let _ = handle.join(); // the panic is caught here, not silently swallowed

match jobs_completed.lock() { // <- returns Err: the Mutex is now marked poisoned
    Ok(count) => println!("{count} jobs completed"),
    Err(poisoned) => println!("recovering after a worker panic: {}", *poisoned.into_inner()),
}
```

**Why this way:** the
[`std::sync::Mutex` docs](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
describe poisoning as turning a potential silent-corruption bug into an
explicit `Err` a caller must decide how to handle — even Rust's failure
mode for a threading bug is a compile-time-typed signal, not an
undiagnosed data race.

## Embedded Rust Notes

**Full support** for the underlying guarantee — `Send`/`Sync` and the
borrow checker are `core`-level and cost nothing at runtime regardless of
target. If anything, compile-time data-race rejection matters *more* on
embedded targets: there's rarely a debugger, sanitizer, or crash reporter
attached to firmware in the field, so catching a race between a main loop
and an interrupt handler at compile time — rather than during an
unreproducible field failure — is disproportionately valuable.
`std::thread` itself needs an OS and isn't available under `#![no_std]`;
concurrent tasks there come from an RTOS's own primitives or an async
executor like `embassy`, and shared mutable state typically goes through a
`critical-section`-based mutex instead of `std::sync::Mutex` (see
[shared-state concurrency](../concurrency-async/shared-state-concurrency.md)
for that substitution) — but the same `Send`/`Sync` compile-time checking
still applies to every one of those alternatives.
