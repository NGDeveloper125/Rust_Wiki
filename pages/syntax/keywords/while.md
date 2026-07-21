---
title: "while"
kind: keyword
embedded_support: full
groups: ["Control Flow", Basics]
related_concepts: [Ownership]
related_syntax: [loop, for, break, continue]
see_also: [loop, for]
---

## Explanation

`while` repeats a block for as long as a condition remains `true`.

Like `if`, the condition must be a plain `bool` and is not parenthesized
by convention. Unlike `if`, a `while` loop is an expression that always
evaluates to `()` — it cannot produce a value via `break value;`, because
the loop can end without any `break` ever running (the condition simply
turns false), which would leave the loop with no defined result (contrast
with `loop`, which can `break` with a value precisely because a `break`
is its only way to exit normally).

`while let PATTERN = expr { ... }` is a related but distinct form: it
loops for as long as `expr` continues to match `PATTERN`, re-evaluating
`expr` and testing the match on every iteration — commonly used to drain
an iterator or a channel one item at a time.

A `while` loop can be given a label (`'outer: while ... `) so an inner
`break` or `continue` can target it specifically instead of the nearest
enclosing loop.

## Usage examples

### Looping while a condition holds

```
let mut count = 0;
while count < 10 { // <- repeats the block while the condition is `true`
    count += 1;
}
```

A `while` loop always evaluates to `()` and cannot
produce a value via `break value;`, unlike `loop` — the loop can end
without any `break` running (the condition turns false), so there would
be no value to yield.

### Message passing between threads

Draining whatever messages are currently queued, without blocking the
thread if the channel is momentarily empty, is a `while let` over
`try_recv`.

```
use std::sync::mpsc;

fn drain_pending(rx: &mpsc::Receiver<String>) {
    while let Ok(message) = rx.try_recv() {
        // <- `while` keeps polling as long as a message is immediately available
        println!("received: {message}");
    }
    // falls through here once nothing is queued right now (the channel isn't necessarily closed)
}
```

[`Receiver::try_recv`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.try_recv)
never blocks, so the `while` condition simply stops being true once the
queue is momentarily empty — contrast with `for message in rx` (see
[`for`](for.md)), which blocks until the channel closes entirely.

### Validating input

Retrying against a work queue of candidate values until one parses
successfully is a `while let Some(...)` loop popping the next candidate
each iteration.

```
let mut pending = vec!["abc".to_string(), "".to_string(), "42".to_string()];
let mut valid_port = None;

while let Some(entry) = pending.pop() {
    // <- keeps pulling candidates until one parses or the queue is empty
    if let Ok(port) = entry.trim().parse::<u16>() {
        valid_port = Some(port);
        break;
    }
}
```

`while let Some(...) = queue.pop()` is the general
shape for "keep trying until the source runs out or a condition is met"
when the source isn't an iterator (over an iterator, a `for` loop is the
idiomatic form — Clippy's `while_let_on_iterator` lint suggests exactly
that rewrite) — the
[Reference's loop expressions](https://doc.rust-lang.org/reference/expressions/loop-expr.html)
page defines exactly this semantics for `while let`.

## Explanation (Embedded)

`while` maps directly onto the most common polling idiom in bare-metal
code: "spin while this status bit is not yet set." It's identical under
`#![no_std]` — a plain condition check and jump, no allocator or OS
involvement. One thing worth flagging that doesn't come up in hosted
code: an unbounded `while` polling a register that never sets — because
the peripheral is disconnected, misconfigured, or dead — hangs the
program forever, and unlike a hosted program a user can just kill from a
terminal, a wedged firmware loop usually means a full power-cycle to
recover. Any polling `while` that isn't guaranteed to terminate by
hardware behavior alone is worth pairing with an attempt counter or a
hardware timer as a timeout.

## Usage examples (Embedded)

### Polling a status bit until it sets

```
while !uart.status().read().txe().bit_is_set() {} // <- spins while the transmit-empty flag is not yet set
uart.data().write(|w| w.bits(byte as u32));
```

### Polling with a timeout

```
fn wait_ready(sensor: &Sensor, max_attempts: u32) -> Result<(), &'static str> {
    let mut attempts = 0;
    while !sensor.is_ready() && attempts < max_attempts {
        // <- bounded: an unbounded `while` here would hang forever if the sensor is disconnected
        attempts += 1;
    }
    if sensor.is_ready() { Ok(()) } else { Err("sensor timed out") }
}
```
