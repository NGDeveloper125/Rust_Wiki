---
title: "loop"
kind: keyword
embedded_support: full
groups: ["Control Flow", Basics]
related_concepts: [Expression-oriented language]
related_syntax: [while, for, break, continue]
see_also: [while, break]
---

## Explanation

`loop` repeats a block unconditionally, forever, until a `break` inside it
(or its body) is reached.

Unlike `while` and `for`, `loop` **is** an expression that can produce a
value — `break value;` exits the loop and evaluates the whole `loop` to
`value`. This is possible precisely because the compiler can see there's no
"falling off the end without a value" case to reconcile, the way there is
with `while`/`for` (which might run zero iterations). A `loop` with no
`break` at all has type `!` (never) — the compiler knows control can never
leave it normally, which is useful for things like a server's main event
loop.

Like other loops, `loop` accepts a label (`'a: loop { ... }`) so nested
`break`/`continue` can target a specific enclosing loop.

## Usage examples

### Producing a value from a loop with `break`

```
let result = loop { // <- `loop` repeats the block forever until a `break`
    break 5;
};
```

### Multi-threading

A worker thread that services jobs from a channel but also needs to do
periodic housekeeping between jobs is a classic use for `loop` plus
`match`: there is no natural boolean condition to test, only distinct
events — a job arriving, a quiet period, the channel closing.

```
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn spawn_worker(rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // <- `loop` is the worker's unconditional service loop
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(job) => println!("processing {job}"),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    println!("no work yet -- heartbeat / housekeeping");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break, // channel closed, all senders dropped
            }
        }
    })
}
```

`recv_timeout` gives the loop three genuinely distinct
events — a job, a timeout for housekeeping, and channel shutdown — which
no single `while` condition (or `for job in rx`) can express; the only
exit is the channel closing. This extends the channel-receiving patterns
in the
[Book's message-passing chapter](https://doc.rust-lang.org/book/ch16-02-message-passing.html);
note that when the only cases are "got a job" and "channel closed",
`for job in rx` is preferred — Clippy's `while_let_loop` lint flags the
plain `loop`/`recv`/`match` version of that.

### Message passing between threads

Polling a channel without blocking the thread — instead of calling the
blocking `.recv()` — pairs `loop` with `try_recv`'s two distinct error
cases: nothing available yet, or the channel is gone for good.

```
use std::sync::mpsc;
use std::time::Duration;
use std::thread;

let (tx, rx) = mpsc::channel::<u32>();

thread::spawn(move || {
    for reading in [17, 18, 19] {
        tx.send(reading).unwrap();
        thread::sleep(Duration::from_millis(20));
    }
}); // `tx` drops here, so the loop below eventually sees `Disconnected`

loop {
    // <- `loop` polls the channel repeatedly instead of blocking on `recv`
    match rx.try_recv() {
        Ok(reading) => println!("sensor reading: {reading}"),
        Err(mpsc::TryRecvError::Empty) => thread::sleep(Duration::from_millis(50)),
        Err(mpsc::TryRecvError::Disconnected) => break,
    }
}
```

[`Receiver::try_recv`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.try_recv)
distinguishes "empty for now" from "disconnected," which a blocking
`recv()` inside `loop` can't do — the sleep keeps this from becoming a
busy-spin that pegs a CPU core while waiting.

## Explanation (Embedded)

`loop` is arguably more central in embedded Rust than anywhere else in the
language. A bare-metal firmware image has no OS to return control to when
`main` finishes — there is no "exit," only "keep running or halt" — so
`fn main() -> !` almost always ends with an unconditional `loop { ... }`
as its body, and the `!` return type documents at the signature level
that this function is never expected to return. The same shape appears
one level down, too: before an interrupt-driven design is wired up (or
when one genuinely isn't warranted), the standard way to wait on a
peripheral is a tight `loop` that polls a status bit and `break`s once
it's set — a busy-wait. Because `loop` is the one loop form that can
`break` with a value, that pattern doubles as "spin until ready, then
hand back what you were waiting for" in a single expression.

## Usage examples (Embedded)

### The firmware entry point that never returns

```
#[entry]
fn main() -> ! {
    let mut led = configure_led();
    loop {
        // <- `loop {}` is the idiomatic never-returning body of an embedded main function
        led.toggle();
        delay.delay_ms(500u16);
    }
}
```

### Busy-waiting on a peripheral flag before interrupts are wired up

```
loop {
    // <- polls the status register directly; a real driver would switch to an interrupt once one is configured
    if uart.status().read().txe().bit_is_set() {
        break;
    }
}
uart.data().write(|w| w.bits(byte as u32));
```
