---
title: "Async/await"
area: "Concurrency & Async"
embedded_support: partial
groups: ["Concurrency & Async", "Concurrent / Message-Passing", "Writing Async Code", "Async I/O"]
related_syntax: [async, await, move]
see_also: ["Futures", "Async runtimes", "Threads (std::thread)", "Send & Sync"]
---

## Explanation

`async`/`.await` is Rust's syntax for cooperative, non-blocking
concurrency: an `async fn` doesn't run its body when called — it returns
a value implementing [`Future`](futures.md) that represents the work,
still not started, and `.await` is what actually drives that future
toward completion, suspending the current async function at that point
until the awaited future has a result. Where an OS thread blocks the
whole thread while it waits on I/O, an `.await` point only suspends the
current *task*; whatever's driving the futures is free to make progress
on other tasks in the meantime. This lets a single OS thread juggle
thousands of concurrent, mostly-waiting tasks — a network server holding
open ten thousand slow client connections — far more cheaply than one
[thread](threads.md) per connection ever could.

Under the hood, the compiler transforms an `async fn` into an anonymous
state machine type that implements `Future`: each `.await` point becomes a
state the machine can be suspended and resumed from, and local variables
that need to live across an `.await` become fields of that generated
type. None of this machinery is visible in the source — writing
`async`/`.await` reads like ordinary sequential code, but compiles into
something that yields control back to whatever is driving it instead of
blocking. That "whatever is driving it" is deliberately not part of the
language: `async`/`.await` are core-language syntax, but actually
executing a future to completion is the job of an [async
runtime](async-runtimes.md) like `tokio`, which polls it repeatedly until
it's done.

Because suspension only happens at explicit `.await` points, an async
function is cooperative in a very literal sense: it must yield control
back voluntarily, and a task that never reaches an `.await` — say, one
doing a long CPU-bound computation — will starve every other task
sharing the same executor thread. This is the sharp edge async/await
carries that threads don't: "never block inside an async fn" is a rule
enforced by discipline and runtime-provided escape hatches (like
`tokio::task::spawn_blocking`), not by the compiler.

`async`/`.await` and threads aren't competitors so much as tools for
different jobs: async excels at *I/O-bound* concurrency — many tasks
mostly waiting on the network, a disk, or a timer — while
[threads](threads.md) (or `spawn_blocking` from within async code) remain
the right tool for *CPU-bound* parallel work. Real async programs
routinely combine both, moving genuinely blocking or CPU-heavy work off
onto a thread rather than letting it starve the async task scheduler.

## Basic usage example

```
async fn fetch_greeting() -> String { // <- async fn: calling it returns a Future, body doesn't run yet
    String::from("hello")
}

async fn greet() {
    let greeting = fetch_greeting().await; // <- .await suspends `greet` until the inner future resolves
    println!("{greeting}");
}
```

**Note:** this compiles on its own, but nothing above actually *runs*
`greet`'s body — an `async fn` only produces a `Future`; driving it to
completion needs an [async runtime](async-runtimes.md) such as `tokio`.

## Best practices & deeper information

### Scenario: Async tasks

A monitoring service polling several independent sensors should spawn one
async task per sensor rather than awaiting them one after another, so a
slow sensor doesn't hold up the others.

```
// [dependencies] tokio = { version = "1", features = ["full"] }
use std::time::Duration;

async fn read_sensor(id: u32) -> f64 { // <- async fn: returns a Future when called
    tokio::time::sleep(Duration::from_millis(50)).await; // <- suspends this task, frees the thread meanwhile
    21.0 + id as f64 * 0.1
}

#[tokio::main]
async fn main() {
    let mut tasks = Vec::new();
    for id in 0..3 {
        tasks.push(tokio::spawn(read_sensor(id))); // <- each sensor read runs as its own concurrent task
    }

    for task in tasks {
        let reading = task.await.unwrap(); // <- .await here waits only for this task's result
        println!("sensor reading: {reading:.1}");
    }
}
```

**Why this way:** spawning each sensor read as its own task lets the
runtime make progress on all three concurrently instead of serializing
them behind sequential `.await`s, which is exactly the concurrency
`tokio::spawn` exists to provide — the
[Tokio tutorial](https://tokio.rs/tokio/tutorial/spawning) covers spawning
independent tasks as the standard way to run async work concurrently.

### Scenario: Handling and propagating errors

An async function that parses a configuration value can use the `?`
operator exactly like a synchronous one — `async`/`.await` doesn't change
how errors propagate, it only changes how the function's control flow
suspends.

```
use std::num::ParseIntError;

async fn parse_timeout(raw: &str) -> Result<u32, ParseIntError> {
    let timeout: u32 = raw.parse()?; // <- `?` propagates the parse error, same as outside async
    Ok(timeout)
}

async fn load_settings(raw_timeout: &str) -> Result<u32, ParseIntError> {
    let timeout = parse_timeout(raw_timeout).await?; // <- await the Future, then `?` the Result it yields
    Ok(timeout)
}
```

**Why this way:** `.await` simply unwraps the `Future` to get its output,
so chaining `.await?` composes exactly like calling a synchronous
fallible function — the
[Rust Book's async chapter](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html)
shows `?` working unchanged inside `async fn` bodies for this reason.

## Embedded Rust Notes

**Partial support.** `async fn` and `.await` are language features that
desugar to a state machine implementing `core::future::Future`, which
needs neither `std` nor an allocator for the syntax itself — this works
under `#![no_std]`. What changes on embedded targets is what drives that
future: there's no `tokio`, so `#![no_std]` async code relies on an
alloc-based (or allocation-free) embedded executor, most commonly
`embassy`, which provides its own `#[embassy_executor::main]` entry point
and async-friendly peripheral drivers built around the same `.await`
syntax shown above.
