---
title: "loop"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language]
related_syntax: [while, for, break, continue]
see_also: [while, break]
---

## Explanation

`loop` repeats a block unconditionally, forever, until a `break` inside it
(or its body) is reached:

```
loop {
    if done {
        break;
    }
}
```

Unlike `while` and `for`, `loop` **is** an expression that can produce a
value — `break value;` exits the loop and evaluates the whole `loop` to
`value`:

```
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2;
    }
};
```

This is possible precisely because the compiler can see there's no
"falling off the end without a value" case to reconcile, the way there is
with `while`/`for` (which might run zero iterations). A `loop` with no
`break` at all has type `!` (never) — the compiler knows control can never
leave it normally, which is useful for things like a server's main event
loop.

Like other loops, `loop` accepts a label (`'a: loop { ... }`) so nested
`break`/`continue` can target a specific enclosing loop.

## Basic usage example

```
let result = loop { // <- `loop` repeats the block forever until a `break`
    break 5;
};
```

## Best practices & deeper information

### Scenario: Multi-threading

A worker thread that services jobs from a channel for its entire
lifetime is the classic use for `loop`: it has no natural boolean
condition to test, only an event (the channel closing) that ends it.

```
use std::sync::mpsc;
use std::thread;

fn spawn_worker(rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // <- `loop` is the worker's unconditional service loop
            match rx.recv() {
                Ok(job) => println!("processing {job}"),
                Err(_) => break, // channel closed, all senders dropped
            }
        }
    })
}
```

**Why this way:** a `loop` with no `break` has type `!`, and here the only
exit is the channel closing — this shape is exactly what the
[Book's threads chapter](https://doc.rust-lang.org/book/ch16-01-threads.html)
uses for a long-lived thread that runs until its work source is gone,
rather than a `while` needing an artificial condition to test.

### Scenario: Message passing between threads

Polling a channel without blocking the thread — instead of calling the
blocking `.recv()` — pairs `loop` with `try_recv`'s two distinct error
cases: nothing available yet, or the channel is gone for good.

```
use std::sync::mpsc;
use std::time::Duration;
use std::thread;

let (tx, rx) = mpsc::channel::<u32>();

loop {
    // <- `loop` polls the channel repeatedly instead of blocking on `recv`
    match rx.try_recv() {
        Ok(reading) => println!("sensor reading: {reading}"),
        Err(mpsc::TryRecvError::Empty) => thread::sleep(Duration::from_millis(50)),
        Err(mpsc::TryRecvError::Disconnected) => break,
    }
}
```

**Why this way:** [`Receiver::try_recv`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.try_recv)
distinguishes "empty for now" from "disconnected," which a blocking
`recv()` inside `loop` can't do — the sleep keeps this from becoming a
busy-spin that pegs a CPU core while waiting.

## Embedded Rust Notes

**Full support.** An unconditional `loop {}` is the idiomatic body of
`fn main() -> !` in almost every embedded firmware entry point, since a
bare-metal program never "exits" the way a hosted process does.
