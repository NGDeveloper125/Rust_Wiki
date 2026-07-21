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

## Explanation (Embedded)

The `async`/`.await` language mechanics don't change under `#![no_std]` —
an `async fn` still desugars to a state machine implementing
`core::future::Future`, and `.await` still just suspends the current task
at that point. What's missing is the other half of the story that makes
async worth reaching for at all: on a hosted target, `.await`ing a socket
read or a sleep only avoids blocking a thread because `tokio` (or
`async-std`) is underneath it, backed by an OS scheduler, OS threads, and
an OS I/O reactor. None of that exists on a microcontroller with no
operating system, so bare `async fn`/`.await` compiles under `#![no_std]`
but has nothing to drive it — the caveat this page's `partial` support
marks.

The idiomatic substitute is `embassy`, an async executor built specifically
for `no_std` embedded targets: `embassy_executor` provides the task
scheduler and `#[embassy_executor::main]` entry point that `tokio` and
`#[tokio::main]` provide on a hosted target, and crates like `embassy_time`
provide `.await`-able timers in place of `tokio::time::sleep`. It's worth
being honest about *why* async earns its keep in embedded, because it's a
narrower case than the network-server story that motivates it on a hosted
target: there's no thread pool to save context-switch overhead on, since
many microcontrollers only have one core and no threads at all. The real
win is letting one core interleave several genuinely independent
wait-heavy jobs — waiting on a UART line, a sensor's timer-driven poll
interval, a button debounce — without either busy-polling every peripheral
in a single loop (wasting power and blurring each job's logic together) or
pulling in a full RTOS just to get preemptive multitasking. Async gives
cooperative multitasking with plain `.await` points, at the cost of the
same discipline classic async requires: a task that never reaches an
`.await` (a long computation, or a blocking HAL call) starves every other
task on embassy's executor just as it would on `tokio`'s.

## Basic usage example (Embedded)

```
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
async fn blink() { // <- async fn: same grammar as hosted Rust, driven by embassy's executor instead of tokio's
    loop {
        // ... toggle an LED pin here
        Timer::after(Duration::from_millis(500)).await; // <- suspends only this task; the executor runs others meanwhile
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(blink()).unwrap();
}
```

## Best practices & deeper information (Embedded)

### Scenario: Async tasks

A device that needs to sample a sensor on a timer while also watching a
button for a debounced press should run both as separate embassy tasks
rather than interleaving the two jobs by hand in one loop — each task
just `.await`s the event it cares about, and embassy's executor handles
interleaving them.

```
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
async fn sample_sensor() { // <- async fn: one independent job, expressed as straight-line code
    loop {
        // ... trigger an ADC read here
        Timer::after(Duration::from_millis(200)).await; // <- suspends this task only
    }
}

#[embassy_executor::task]
async fn watch_button() {
    loop {
        // ... await a GPIO edge future here (debounced by the HAL/driver)
        Timer::after(Duration::from_millis(20)).await; // <- suspends this task only
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(sample_sensor()).unwrap(); // <- both tasks run concurrently on one core, no RTOS involved
    spawner.spawn(watch_button()).unwrap();
}
```

**Why this way:** writing the sensor-sampling and button-watching logic as
two separate `async fn` tasks keeps each one readable as sequential code,
while embassy's executor multiplexes them on the single core whenever one
task is suspended at an `.await` — the alternative, hand-rolling a
single loop that polls both jobs, quickly turns into an ad-hoc scheduler
that async/`.await` exists to avoid writing.

### Scenario: Multi-threading

A firmware image that has both a CPU-bound task (a DSP filter running
every sample) and several I/O-wait-bound tasks (sensor polling, a UART
command parser) shouldn't force the CPU-bound work onto the same
cooperative executor as the waiting tasks — that's the one case embedded
async doesn't help with, and it's still a job for a thread (or the second
core, on a dual-core target) rather than another embassy task.

```
// AVOID: a CPU-bound loop inside an async task never reaches an `.await`,
// so it starves every other task on embassy's single-threaded executor.
#[embassy_executor::task]
async fn run_filter_bad(samples: [i16; 256]) {
    let _ = heavy_dsp_compute(&samples); // <- no `.await` anywhere: blocks the whole executor until done
}

// PREFER: run genuinely CPU-bound work on its own thread/core (e.g. via a
// second core's executor, or an RTOS task) and hand results back through
// a channel, keeping the async executor free to service waiting tasks.
fn heavy_dsp_compute(samples: &[i16; 256]) -> i16 {
    samples.iter().copied().sum::<i16>() / samples.len() as i16
}
```

**Why this way:** embassy's default executor is cooperative and typically
single-threaded per core, so it inherits the same rule hosted async
lives by — never block inside an async task — except here "block" also
includes a long computation with no `.await` in it; genuinely CPU-bound
work belongs on a thread or a dedicated core, exactly as `spawn_blocking`
carves it out of `tokio`'s executor on a hosted target.
