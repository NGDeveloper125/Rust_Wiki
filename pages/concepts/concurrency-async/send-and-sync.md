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

## Explanation (Embedded)

`Send` and `Sync` are defined in `core::marker`, not `std`, so nothing
about their meaning changes under `#![no_std]` — this is a `full`-support
page precisely because the marker-trait mechanics described above are
identical on a microcontroller. What's genuinely different is *where*
concurrency shows up to make these traits load-bearing. There's usually no
OS scheduler and no `std::thread::spawn` on bare metal, but there is
almost always an interrupt controller: firmware routinely shares state
between `fn main`'s loop and one or more `#[interrupt]` handlers, and an
interrupt handler is, for `Send`/`Sync` purposes, a second concurrent
context in exactly the same sense a second OS thread is — it can run
"at the same time" as the main loop from the compiler's point of view,
because it can preempt the main loop at any instruction boundary.

Because embedded code frequently runs with no heap and no `Arc`/`Mutex`
from `std`, the idiomatic shared-state type is different, but the question
`Send`/`Sync` answers is the same one: is it safe for two contexts (main
loop and ISR, or two async tasks) to touch this data? The `critical-section`
crate is the closest embedded analog to `std::sync::Mutex` — it provides a
`Mutex<T>` wrapper that is `Sync` only when `T: Send`, and only allows
accessing the wrapped value inside a critical section (typically
interrupts-disabled on a single-core target), which is what makes it sound
to hand a `'static` reference to that `Mutex` to both `main` and an
interrupt handler. Async executors like `embassy` add a third concurrent
context — cooperatively scheduled tasks — and the same `Send` bound shows
up again on anything crossing an `.await` point or being moved into
`spawner.spawn(...)`.

## Basic usage example (Embedded)

```
use core::cell::RefCell;
use critical_section::Mutex;

// Sync only because RefCell<u32> — via u32 — is Send; critical-section's
// Mutex is what makes the RefCell safe to share with an interrupt handler.
static COUNTER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0)); // <- Mutex<RefCell<u32>>: Sync, shareable with an ISR

fn on_tick_interrupt() {
    critical_section::with(|cs| {
        let mut counter = COUNTER.borrow_ref_mut(cs);
        *counter += 1;
    });
}
```

## Best practices & deeper information (Embedded)

### Scenario: Sharing state across threads

A tick counter incremented by a timer interrupt and read from the main
loop needs a type that's genuinely `Sync` for a single-core target with no
OS — `critical-section::Mutex` fills the role `std::sync::Mutex` plays on
a hosted target, without needing a thread to block on.

```
use core::cell::RefCell;
use critical_section::Mutex;

static TICKS: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0)); // <- static requires TICKS: Sync, checked at compile time

#[interrupt]
fn TIM2() { // <- runs as a second concurrent context alongside `main`
    critical_section::with(|cs| {
        *TICKS.borrow_ref_mut(cs) += 1;
    });
}

fn main() -> ! {
    loop {
        let ticks = critical_section::with(|cs| *TICKS.borrow(cs));
        if ticks % 1000 == 0 {
            // ... report the tick count
        }
    }
}
```

**Why this way:** a plain `RefCell<u32>` is `!Sync`, so `static TICKS:
RefCell<u32>` would fail to compile the instant an interrupt handler tried
to touch it too — wrapping it in `critical-section`'s `Mutex` is what
makes the type `Sync` and forces every access through a critical section,
the same discipline `std::sync::Mutex` enforces for OS threads, applied
here with no OS underneath it.

### Scenario: Designing a public API

A driver crate exposing a handle to a peripheral register block should
leave that handle `!Send`/`!Sync` unless the author has verified it's
safe to move or share across the main-loop/interrupt boundary — the same
API-design discipline that applies to native OS handles applies to raw
peripheral access.

```
use core::marker::PhantomData;

pub struct SpiHandle {
    base_addr: usize,
    _not_sync: PhantomData<*const ()>, // <- opts out of auto-Send/Sync, same as a native-handle wrapper would
}

impl SpiHandle {
    /// # Safety
    /// Caller must guarantee no other `SpiHandle` aliases this peripheral.
    pub unsafe fn at(base_addr: usize) -> Self {
        SpiHandle { base_addr, _not_sync: PhantomData }
    }
}
// No `unsafe impl Send`/`Sync` here: touching the register block from both
// `main` and an interrupt handler without synchronization would race.
```

**Why this way:** defaulting to `!Send`/`!Sync` and requiring a deliberate
`unsafe impl` once the driver author has actually reasoned about
interrupt-safety is exactly the [API Guidelines'
C-SEND-SYNC](https://rust-lang.github.io/api-guidelines/interoperability.html)
checklist item, applied to the main-loop/interrupt boundary instead of an
OS thread boundary — the risk (silent auto-derived thread-safety for a
type that secretly isn't) is identical.
