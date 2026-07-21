---
title: "Futures"
area: "Concurrency & Async"
embedded_support: partial
groups: ["Concurrency & Async", "Concurrent / Message-Passing", "Writing Async Code", "Async I/O"]
related_syntax: [async, await]
see_also: ["Async/await", "Async runtimes", "Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

A future is a value representing work that hasn't finished yet — the
async equivalent of a promise or deferred computation. Concretely, it's
any type implementing the standard library's `Future` trait, whose single
method, `poll`, is asked "are you done?" and answers either `Poll::Ready(value)`
or `Poll::Pending`. [`async`/`.await`](async-await.md) is almost always how
futures get created and driven in ordinary code — an `async fn` returns an
anonymous type implementing `Future`, and `.await` repeatedly polls it —
but the trait itself is the lower-level abstraction that syntax compiles
down to, and the vocabulary everything else in async Rust (combinators,
executors, runtimes) is built around.

The most important thing to internalize about futures is that they are
*lazy*: creating one does nothing on its own. Calling an `async fn` builds
the state machine but runs none of its body — work only happens when
something polls the future, and nothing polls it unless it's `.await`ed or
handed to an executor. This is unlike a thread, which starts running the
moment it's spawned; a future sitting in a variable, never awaited or
passed to `tokio::spawn`, simply never executes and (in a debug build)
Rust will even warn that it "must be used." This laziness is deliberate:
it lets futures be built up, combined, and passed around as inert values
before any work commits to running, which is what makes combinators like
"race these two futures" or "run these three concurrently" possible
without spawning anything ahead of time.

Because polling is how a future makes progress, something has to be
responsible for calling `poll` — repeatedly, and only when there's a
reasonable chance progress can be made. That's the job of an [async
runtime](async-runtimes.md): a future on its own is inert data, not a
running computation, and the runtime's executor is what actually drives
it forward, using the waker mechanism built into `poll`'s `Context` to
know when a pending future is worth polling again (e.g. once the socket it
was waiting on has data). This split — futures as passive descriptions of
work, executors as the thing that runs them — mirrors the split between
[`Send`/`Sync`](send-and-sync.md) as safety guarantees and threads as the
thing that actually runs code across cores: the trait defines the shape,
something else provides the engine.

Most everyday async code never touches the `Future` trait directly —
`async fn` and `.await` cover the common case — but the trait becomes
visible at API boundaries: a function that needs to return "some future"
without picking a concrete executor writes `-> impl Future<Output = T>`,
and code that needs to store futures of different concrete types in one
collection (a scheduler holding heterogeneous pending tasks) reaches for
`Pin<Box<dyn Future<Output = T> + Send>>`, the async analog of [trait
objects](../traits-polymorphism/trait-objects-dynamic-dispatch.md) for
dynamic dispatch over otherwise-unrelated future types.

## Basic usage example

```
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Immediately<T>(Option<T>); // <- a minimal Future: ready the very first time it's polled

impl<T: Unpin> Future for Immediately<T> { // <- implementing the Future trait directly, no async fn involved
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        Poll::Ready(self.0.take().expect("polled after completion"))
    }
}
```

**Note:** this compiles standalone because it only *defines* a `Future` —
actually polling `Immediately` to get its value still needs something to
call `poll`, whether that's `.await` inside an `async fn` or an [async
runtime](async-runtimes.md)'s executor.

## Best practices & deeper information

### Scenario: Async tasks

A dashboard that needs three independent readings before it can render
should drive all three futures concurrently with `tokio::join!`, rather
than awaiting them one at a time and paying their latencies sequentially.

```
// [dependencies] tokio = { version = "1", features = ["full"] }
use std::time::Duration;

async fn fetch_cpu_load() -> f64 {
    tokio::time::sleep(Duration::from_millis(30)).await;
    0.42
}

async fn fetch_memory_used() -> f64 {
    tokio::time::sleep(Duration::from_millis(20)).await;
    0.71
}

#[tokio::main]
async fn main() {
    // <- tokio::join! polls both futures concurrently, resolving once both are Ready
    let (cpu, memory) = tokio::join!(fetch_cpu_load(), fetch_memory_used());
    println!("cpu={cpu:.2} memory={memory:.2}");
}
```

**Why this way:** because futures are inert until polled, `tokio::join!`
can hold both of them and poll each whenever it's worth doing so, finishing
in roughly the time of the slower reading instead of the sum of both — the
[Tokio tutorial](https://tokio.rs/tokio/tutorial/select#join) presents
`join!` as the standard way to run independent futures to completion
concurrently within one task.

### Scenario: Designing a public API

A library function that performs async work but doesn't want to commit
callers to a specific executor should return `impl Future` from its
signature; a scheduler that needs to store many different concrete future
types together has to erase that type with a boxed trait object instead.

```
use std::future::Future;
use std::pin::Pin;

fn delayed_greeting(name: String) -> impl Future<Output = String> { // <- concrete type hidden, still zero-cost
    async move { format!("hello, {name}") }
}

struct Scheduler {
    // <- boxed and pinned: the only way to store futures of different concrete types in one Vec
    pending: Vec<Pin<Box<dyn Future<Output = String> + Send>>>,
}

impl Scheduler {
    fn push(&mut self, fut: impl Future<Output = String> + Send + 'static) {
        self.pending.push(Box::pin(fut));
    }
}
```

**Why this way:** `impl Future` keeps the zero-cost, statically-dispatched
type whenever the caller only needs one concrete future, while
`Pin<Box<dyn Future<...>>>` is the standard way to erase that type when
heterogeneous futures need to live together in one collection — the
[`std::future::Future` docs](https://doc.rust-lang.org/std/future/trait.Future.html)
note this boxed-trait-object pattern as the usual answer when a fixed
concrete `Future` type isn't expressible.

## Explanation (Embedded)

The `Future` trait itself is core-language: it lives in `core::future`, not
`std::future`, so defining a type that implements `poll` and returning
`Poll::Ready`/`Poll::Pending` works identically under `#![no_std]` — a
future is just as lazy and just as much an inert value on a
microcontroller as on a hosted target. That's the `full`-equivalent half
of the story; the `partial` caveat is entirely about the other half: a
future is only ever *data describing* work, and something external has to
call `poll` on it, repeatedly, for that work to actually happen. On a
hosted target that "something" is an executor like `tokio`'s, built on an
OS scheduler and an OS I/O reactor that wakes tasks when their socket or
timer is ready. Bare metal has no OS underneath it to provide any of that,
so `#![no_std]` code that only defines a `Future` compiles fine but never
runs on its own.

The practical answer is the same one the sibling Async/await page reaches:
an embedded async executor such as `embassy` provides the missing engine,
polling `core::future::Future`s the same way `tokio` does, just built to
run without an OS — typically single-threaded per core, woken by
interrupts rather than by an OS-level I/O reactor. It's worth being modest
here rather than asserting specifics: `embassy`'s waking mechanism is
tied to its own timer queue and interrupt-driven peripheral drivers, and
the exact plumbing is an implementation detail this page doesn't need to
get into — the load-bearing fact is only that *some* executor has to
exist to drive a future to completion, and on `no_std` that executor is
not `tokio`.

## Basic usage example (Embedded)

```
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

struct ReadyNow<T>(Option<T>); // <- a minimal Future, no_std-compatible: core::future::Future only

impl<T: Unpin> Future for ReadyNow<T> { // <- same trait as on a hosted target, imported from core instead of std
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        Poll::Ready(self.0.take().expect("polled after completion"))
    }
}
```

**Note:** this compiles under `#![no_std]` as-is because it only defines a
`Future` — actually polling `ReadyNow` to get its value still needs a
driver, whether that's `.await` inside an executor-spawned task (e.g. an
`embassy_executor::task`) or a hand-rolled minimal executor.

## Best practices & deeper information (Embedded)

### Scenario: Async tasks

A sensor task that needs to wait on both a timer and an interrupt-driven
data-ready signal should combine both as futures polled together, rather
than blocking on one before checking the other, so a slow timer doesn't
delay noticing the sensor is already ready.

```
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

async fn wait_for_reading() -> u16 {
    Timer::after(Duration::from_millis(50)).await; // <- an embassy-provided Future; this task only owns the .await point
    // ... read the conversion result register here
    512
}

#[embassy_executor::task]
async fn sample_loop() {
    loop {
        let reading = wait_for_reading().await; // <- polling happens inside embassy's executor, not this code
        // ... use `reading`
        let _ = reading;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(sample_loop()).unwrap();
}
```

**Why this way:** because `wait_for_reading`'s future is inert until
something polls it, embassy's executor is free to poll it only when the
underlying timer future indicates progress is possible, rather than this
code having to busy-poll a register itself — the same lazy-until-polled
property that makes `tokio::join!` work concurrently on a hosted target is
what lets embassy avoid wasting cycles here.

### Scenario: Designing a public API

A sensor-driver crate that performs async reads but shouldn't force
callers onto a specific embedded executor should return `impl Future`
from its API, the same choice a hosted async library makes to stay
executor-agnostic.

```
use core::future::Future;

pub fn read_temperature() -> impl Future<Output = i16> { // <- concrete future type hidden, still no_std, still zero-cost
    async {
        // ... trigger a conversion and await its completion
        21
    }
}
```

**Why this way:** returning `impl Future<Output = i16>` keeps the driver
usable from any executor that can poll a `core::future::Future` —
`embassy` or a hand-rolled one — instead of hard-coding a dependency on
one specific executor's task-spawning API, the same executor-agnostic
reasoning the classic Explanation applies to `impl Future` on a hosted
target.
