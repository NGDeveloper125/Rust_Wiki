---
title: "async"
kind: keyword
embedded_support: partial
groups: ["Concurrency & Async"]
related_concepts: ["Async/await"]
related_syntax: ["await", "move"]
see_also: ["await", "move"]
---

## Explanation

`async` marks a function or a block as asynchronous, in two distinct
grammatical forms:

1. **`async fn name(params) -> RetType { body }`** — an ordinary function
   declaration with `async` written before `fn`. Calling it does not run
   `body`; it immediately returns a value of an anonymous type
   implementing `Future<Output = RetType>`. The signature is still
   written in terms of the "real" return type (`-> RetType`, not
   `-> impl Future<Output = RetType>`) — the compiler performs that
   wrapping for you. `async fn` is legal as a free function, as an
   inherent method inside `impl`, and, since Rust 1.75, as a trait method
   with a body inside a `trait` block — though a trait containing an
   `async fn` isn't `dyn`-compatible without extra work.
2. **`async { ... }` / `async move { ... }`** — a block *expression*,
   legal anywhere an expression is legal (assigned to a variable,
   returned, passed as an argument), that evaluates to a value of an
   anonymous type implementing `Future<Output = T>`, where `T` is the
   type of the block's own tail expression — exactly like an ordinary
   block, the tail expression (no trailing semicolon) becomes the
   future's output.

The `move` in `async move { ... }` is the same `move` used on closures
(see [`move`](move.md)): it forces the block to take ownership of every
variable it references from the surrounding scope, moving or copying them
into the generated future instead of borrowing them. This matters because
the future an `async` block produces routinely outlives the frame that
created it — handed to `tokio::spawn`, stored in a struct, returned from
the function that built it — so it can't safely hold a borrow into a
frame that may already be gone by the time something polls it. Plain
`async { ... }` (without `move`) borrows its captures instead, and only
compiles when the future is guaranteed not to outlive them. `async move`
has no equivalent on `async fn`: an `fn`'s parameters are already owned
bindings passed in the ordinary way, so there is no separate capture step
for `move` to act on — `async move fn` is not valid syntax.

Calling an `async fn`, or evaluating an `async` block, produces a
**value** — not a running computation. Arguments are evaluated eagerly at
the call site, exactly as with any other function call, but the body
itself does not execute at all until the returned future is driven, via
[`.await`](await.md) or by handing it to an executor. A future sitting
unused in a local variable simply never runs its body.

## Usage examples

### Declaring an `async fn` and an `async move` block

```
async fn fetch_status() -> bool { // <- `async fn`: calling it returns a Future; the body doesn't run yet
    true
}

let flag = true;
let check_flag = async move { flag }; // <- `async move` block: owns `flag` instead of borrowing it
```

### Async tasks

A metrics collector spawns one task per device, using an `async fn` for
the network call itself and a separate `async move` block to pair a
per-device label with that call before handing the whole thing to
`tokio::spawn` — the `move` is what lets the label and id outlive the
loop iteration that created them.

```
// [dependencies] tokio = { version = "1", features = ["full"] }
async fn fetch_uptime_seconds(device_id: u32) -> u64 { // <- `async fn`: returns a Future when called
    device_id as u64 * 100
}

#[tokio::main]
async fn main() {
    let mut tasks = Vec::new();

    for device_id in 0..3 {
        let label = format!("device-{device_id}");
        tasks.push(tokio::spawn(async move { // <- `async move`: block takes ownership of `label` and `device_id`
            let uptime = fetch_uptime_seconds(device_id).await;
            (label, uptime)
        }));
    }

    for task in tasks {
        let (label, uptime) = task.await.unwrap();
        println!("{label}: {uptime}s");
    }
}
```

`tokio::spawn` requires its future to be `'static`, so
any block capturing local variables like `label` must use `async move` to
take ownership of them rather than borrow — an `async fn`'s parameters
need no such annotation because they're already owned by the time the
function body runs, a distinction the
[Tokio tutorial](https://tokio.rs/tokio/tutorial/spawning) calls out
explicitly for spawned tasks.

## Explanation (Embedded)

**Partial support.** `async fn` and `async`/`async move` blocks are
core-language: writing one, having it desugar into an anonymous
`core::future::Future`-implementing state machine, and the distinction
between borrowing (`async` block) and taking ownership (`async move`) of
captured variables all work identically under `#![no_std]` — none of it
depends on `std` or an allocator. What's missing on bare metal is the
other half of the picture the classic examples lean on: `tokio::spawn`,
`#[tokio::main]`, and the hosted runtime that actually polls those
futures to completion. `tokio` is built on an OS scheduler, threads, and
an OS-backed I/O reactor, none of which exist on a microcontroller with
no operating system underneath it.

The real substitute is an embedded async executor — `embassy` is the
most widely used one. Embassy provides its own entry point attribute,
`#[embassy_executor::main]`, in place of `#[tokio::main]`, and its own
task-spawning and timer primitives (`embassy_executor::Spawner`,
`embassy_time::Timer`) in place of `tokio::spawn`/`tokio::time`. An
`async fn` marked `#[embassy_executor::task]` is spawned onto embassy's
executor instead of tokio's, but the `async fn`/`.await` syntax inside it
is exactly the same language feature described in the classic
Explanation above — only the runtime driving it has changed.

## Usage examples (Embedded)

### Declaring an embassy task with `async fn`

```
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
async fn blink_led() { // <- `async fn`: same grammar as hosted Rust, spawned onto embassy's executor instead of tokio's
    loop {
        // ... toggle a GPIO pin here
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) { // <- embassy's entry point, in place of #[tokio::main]
    spawner.spawn(blink_led()).unwrap();
}
```

`spawner.spawn(...)` is embassy's substitute for
`tokio::spawn` — both hand an `async fn`'s produced future to an
executor, just a different one suited to running with no OS underneath
it.
