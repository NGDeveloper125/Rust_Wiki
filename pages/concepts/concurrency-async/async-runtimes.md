---
title: "Async runtimes"
area: "Concurrency & Async"
embedded_support: none
groups: ["Concurrency & Async", "Writing Async Code", "Async I/O"]
related_syntax: [async, await]
see_also: ["Async/await", "Futures", "Threads (std::thread)"]
---

## Explanation

An async runtime is what actually executes [async code](async-await.md):
an executor that polls [futures](futures.md) forward until they complete,
plus a reactor that watches sockets, timers, and files for readiness and
wakes the right task when something changes. Rust's standard library
deliberately ships neither piece — `async`/`.await` and the `Future` trait
are core language features, but *running* a future to completion is left
to a separate crate the programmer chooses. This is a real difference from
languages like JavaScript or Go, where an event loop or goroutine
scheduler is baked into the runtime; in Rust, "what runs my async code"
is an explicit dependency decision, not a given.

The dominant choice for hosted (OS-backed) Rust is `tokio`: a
multi-threaded, work-stealing executor plus an async standard library of
its own — networking, timers, filesystem access, synchronization
primitives — all built to be `.await`-compatible. `async-std` offers a
similar shape with an API closer to `std`'s own naming; `smol` targets a
smaller, more composable core. Almost all of the wider async ecosystem
(`axum`, `sqlx`, `hyper`, `tonic`) is written against `tokio` specifically,
which is why it has become close to a de facto standard for hosted async
Rust even though nothing in the language privileges it.

This split exists on purpose. Not every program needs an async runtime at
all — a CLI tool or a batch job may never touch async code — and forcing
every Rust binary to carry a scheduler and reactor it doesn't use would
work against the language's zero-cost-abstraction philosophy. It also
means the same `async`/`.await` syntax can run under wildly different
runtimes suited to wildly different environments: `tokio` for a
multi-threaded server, `wasm-bindgen-futures` driving a single future per
browser microtask in WebAssembly, or `embassy` polling tasks on a
microcontroller with no operating system at all — none of which would fit
under one runtime built into the standard library.

Picking a runtime is mostly a project-wide, one-time decision rather than
a per-function one: mixing runtimes in the same binary is fragile (a
`tokio`-specific type used from outside a `tokio` runtime panics at
runtime, since the reactor it needs isn't running), so a crate's async API
choices — does it assume `tokio`, or is it runtime-agnostic — matter as
much as its functional behavior when picking a dependency.

## Basic usage example

```
// [dependencies] tokio = { version = "1", features = ["full"] }

#[tokio::main] // <- generates a `fn main()` that starts the tokio runtime, then runs this async body
async fn main() {
    let greeting = async { "hello from an async runtime" }.await;
    println!("{greeting}");
}
```

## Best practices & deeper information

### Scenario: Async tasks

A batch importer that needs to run many I/O-bound downloads concurrently,
plus one CPU-heavy parsing step, should let `tokio`'s multi-threaded
executor schedule the downloads while explicitly moving the CPU-heavy work
off onto a blocking thread so it doesn't stall the async scheduler.

```
// [dependencies] tokio = { version = "1", features = ["full"] }

async fn download(id: u32) -> Vec<u8> {
    // ... network I/O awaited here ...
    vec![0; 1024]
}

fn parse_blocking(data: Vec<u8>) -> usize { // <- plain, synchronous, CPU-bound work
    data.len()
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)] // <- runtime configuration: 4 executor threads
async fn main() {
    let mut downloads = Vec::new();
    for id in 0..8 {
        downloads.push(tokio::spawn(download(id))); // <- runtime schedules these across its worker threads
    }

    for handle in downloads {
        let data = handle.await.unwrap();
        // <- spawn_blocking hands CPU-bound work to a dedicated thread pool, keeping async workers free
        let size = tokio::task::spawn_blocking(move || parse_blocking(data)).await.unwrap();
        println!("parsed {size} bytes");
    }
}
```

**Why this way:** the executor's worker threads exist to keep polling
futures, so a synchronous, CPU-bound function running on one of them would
starve every other task sharing that thread; `spawn_blocking` moves such
work onto tokio's separate blocking-thread pool instead — the
[Tokio tutorial](https://tokio.rs/tokio/tutorial/spawning) calls out never
blocking inside an async task as the central rule its runtime is built
around.

### Scenario: Serving a web endpoint

An axum web server needs a runtime to actually execute its async request
handlers — `#[tokio::main]` is what turns the `Router` into a running
process that accepts connections and drives each handler's future to
completion.

```
// [dependencies] axum = "0.8", tokio = { version = "1", features = ["full"] }
use axum::{routing::get, Router};

async fn health() -> &'static str {
    "ok"
}

#[tokio::main] // <- without a runtime, `app()`'s handlers would just be inert Futures, never polled
async fn main() {
    let app = Router::new().route("/health", get(health));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap(); // <- the runtime drives this future for the server's lifetime
}
```

**Why this way:** axum's handlers are ordinary async functions with no
executor of their own, so `tokio`'s runtime is what turns "a `Router`
describing how to respond" into an actual, long-running server process —
the [axum docs](https://docs.rs/axum/latest/axum/) build every example
around `#[tokio::main]` for exactly this reason.

## Embedded Rust Notes

**No support.** `tokio` (and `async-std`) are built on OS facilities —
epoll/kqueue/IOCP for the reactor, OS threads for the executor's worker
pool — none of which exist on bare-metal embedded targets, so neither
runtime is usable under `#![no_std]`. The embedded world's equivalent is
`embassy`, an async runtime purpose-built for microcontrollers: a
single-threaded, interrupt-driven executor that polls tasks with no heap
allocation required, alongside `embassy-net` and `embassy`'s own
peripheral drivers standing in for what `tokio`'s ecosystem provides on a
hosted OS.
