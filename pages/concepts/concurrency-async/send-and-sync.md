---
title: "Send & Sync"
area: "Concurrency & Async"
embedded_support: full
groups: ["Concurrency & Async", "Concurrent / Message-Passing", "Writing Concurrent & Parallel Code", "Sharing & Mutating Data Safely", "Multithreading"]
related_syntax: [move]
see_also: ["Marker traits (Send, Sync, Sized, Copy)", "Threads (std::thread)", "Shared-state concurrency (Mutex, RwLock)", "Message passing (channels / mpsc)"]
---

## Explanation

`Send` and `Sync` are the two marker traits that answer concurrency's most
basic safety question: is it OK to move this value to another thread
(`Send`), and is it OK for several threads to access it at once through a
shared reference (`Sync`)? Every type in a Rust program is implicitly one,
both, or neither, and the compiler checks these traits as ordinary trait
bounds — a function that spawns a thread with `std::thread::spawn`
requires its closure's captures to be `Send`, so a program that tries to
share an unsafe-to-share type across threads simply fails to compile,
rather than shipping a data race that only shows up under load in
production. See [Marker traits](../traits-polymorphism/marker-traits.md)
for how `Send`/`Sync` fit alongside `Copy` and `Sized` as a category and
how the auto-trait mechanism derives them.

The mental model worth keeping is that `Send` is about *transferring*
(does moving ownership to another thread cause a problem?) and `Sync` is
about *sharing* (does letting two threads hold `&T` at once cause a
problem?) — and that `T: Sync` is precisely equivalent to `&T: Send`. Most
ordinary data is both: an integer, a `String`, a plain struct of `Send`
fields can be freely moved or shared, because nothing about them depends
on which thread touches them. The types that lose one or both are exactly
the ones with some thread-specific invariant baked in — `Rc<T>` is neither
`Send` nor `Sync` because its reference count isn't updated atomically, so
two threads bumping it concurrently would corrupt the count; a raw
pointer isn't `Send` because the compiler can't verify what it points to
or whether aliasing rules still hold once it crosses a thread boundary.

This is what makes Rust's "fearless concurrency" claim more than
marketing: these aren't runtime checks or a linter's opinion, they're
compiler-enforced bounds baked into every API that touches threads. Code
that compiles with threads, channels, or shared locks has already been
checked for an entire class of bugs — sending non-thread-safe data across
threads, or aliasing non-thread-safe data from multiple threads — that in
other systems languages are notorious for surfacing only as intermittent,
hard-to-reproduce crashes.

`Send`/`Sync` rarely need to be written out by hand: because they're auto
traits, a struct or enum gets them automatically whenever every field
already has them, which is the overwhelming majority of the time. They
become visible and load-bearing exactly at the boundaries where
concurrency happens — [spawning a thread](threads.md), [sending a value
through a channel](message-passing-channels.md), or [sharing state behind
a lock](shared-state-concurrency.md) — which is why this page's
explanation leans on those three siblings for its real-world weight rather
than repeating the marker-trait mechanics here.

## Basic usage example

```
use std::rc::Rc;
use std::thread;

fn requires_send<T: Send>(_: T) {}

requires_send(String::from("ready to move")); // <- String is Send: compiles
// requires_send(Rc::new(5)); // would fail to compile: Rc<T> is not Send
```

## Best practices & deeper information

### Scenario: Multi-threading

A connection pool shared across worker threads must be built entirely
from `Send`/`Sync` pieces, or the program simply won't compile once a
second thread tries to touch it — a compile-time guarantee that saves the
worker pool from a whole class of runtime data races.

```
use std::sync::{Arc, Mutex};
use std::thread;

struct ConnectionPool { available: Vec<u32> } // <- auto-Sync: Vec<u32> is Sync, so is the struct

fn spawn_workers(pool: Arc<Mutex<ConnectionPool>>) {
    for worker_id in 0..4 {
        let pool = Arc::clone(&pool); // <- requires ConnectionPool: Send, checked at compile time
        thread::spawn(move || {
            let mut pool = pool.lock().unwrap();
            if let Some(conn) = pool.available.pop() {
                println!("worker {worker_id} using connection {conn}");
            }
        });
    }
}

spawn_workers(Arc::new(Mutex::new(ConnectionPool { available: vec![1, 2, 3] })));
```

**Why this way:** `Arc<Mutex<T>>` is only itself `Send`/`Sync` when `T` is
`Send`, so the compiler statically rules out sharing a pool built from a
non-thread-safe piece before the program ever runs — the
[Rust Book](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)
covers `Send`/`Sync` as the mechanism underpinning every one of the
threading and locking APIs used here.

### Scenario: Sharing state across threads

Choosing `Rc<RefCell<T>>` for a single-threaded cache and switching to
`Arc<Mutex<T>>` the moment that same cache needs to be touched from a
background thread is a decision `Send`/`Sync` makes for you — the compiler
rejects the single-threaded version the instant it crosses a thread
boundary.

```
use std::sync::{Arc, Mutex};
use std::thread;

struct SessionCache { entries: Vec<String> }

fn refresh_in_background(cache: Arc<Mutex<SessionCache>>) {
    thread::spawn(move || { // <- only compiles because Arc<Mutex<SessionCache>> is Send
        cache.lock().unwrap().entries.push("refreshed".into());
    });
}

refresh_in_background(Arc::new(Mutex::new(SessionCache { entries: Vec::new() })));
```

**Why this way:** swapping `Rc`/`RefCell` for `Arc`/`Mutex` is the standard
migration the instant single-threaded shared state needs to cross into a
second thread, and `Send`/`Sync` bounds are exactly what stop the
`Rc`/`RefCell` version from compiling in that context instead of failing
at runtime — the
[Rust Book](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
walks through this same substitution.

### Scenario: Designing a public API

A library wrapping a handle to native, non-thread-safe resources should
leave it `!Send`/`!Sync` by default and only add a manual `unsafe impl`
once the author has actually verified the underlying resource tolerates
that usage.

```
use std::marker::PhantomData;

pub struct DeviceHandle {
    fd: i32,
    _not_sync: PhantomData<*const ()>, // <- opts out of the auto-Send/Sync the compiler would otherwise grant
}

impl DeviceHandle {
    pub fn open(fd: i32) -> Self {
        DeviceHandle { fd, _not_sync: PhantomData }
    }
}
// No `unsafe impl Send for DeviceHandle` here: the underlying device driver
// documents that its handle must only ever be touched from the thread that opened it.
```

**Why this way:** it's far safer for a type to default to `!Send`/`!Sync`
and require a deliberate, documented `unsafe impl` than to let the
compiler auto-derive thread-safety for a type that secretly isn't — the
[API Guidelines' C-SEND-SYNC](https://rust-lang.github.io/api-guidelines/interoperability.html)
treats getting this right as a checklist item precisely because the
auto-trait mechanism otherwise grants `Send`/`Sync` silently.

## Embedded Rust Notes

**Full support.** `Send` and `Sync` are defined in `core::marker`, cost
nothing at runtime, and require no allocator or OS — if anything they
matter more in embedded code than on a hosted target, since they're what
the compiler uses to check that data shared between a main loop and an
interrupt handler, or between tasks on an async executor like `embassy`,
is genuinely safe to share. The same auto-trait rules apply identically
under `#![no_std]`.
