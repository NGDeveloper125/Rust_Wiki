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

## Embedded Rust Notes

**Partial support.** The `Future` trait itself lives in `core::future` and
needs neither `std` nor an allocator, so `#![no_std]` code can implement
and hold futures directly — the trait definition works identically on any
target. What's missing on bare-metal targets is a hosted executor: there's
no `tokio` to poll anything, so day-to-day use of futures in embedded code
depends on an `alloc`-friendly (or fully allocation-free) embedded
executor like `embassy`, which polls `core::future::Future`s the same way
`tokio` does on a hosted OS.
