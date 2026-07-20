---
title: "while"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Ownership]
related_syntax: [loop, for, break, continue]
see_also: [loop, for]
---

## Explanation

`while` repeats a block for as long as a condition remains `true`:

```
let mut count = 0;
while count < 10 {
    count += 1;
}
```

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

## Basic usage example

```
let mut count = 0;
while count < 10 { // <- repeats the block while the condition is `true`
    count += 1;
}
```

**Restriction:** a `while` loop always evaluates to `()` and cannot
produce a value via `break value;`, unlike `loop` — the loop can end
without any `break` running (the condition turns false), so there would
be no value to yield.

## Best practices & deeper information

### Scenario: Message passing between threads

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

**Why this way:** [`Receiver::try_recv`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.try_recv)
never blocks, so the `while` condition simply stops being true once the
queue is momentarily empty — contrast with `for message in rx` (see
[`for`](for.md)), which blocks until the channel closes entirely.

### Scenario: Validating input

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

**Why this way:** `while let Some(...) = queue.pop()` is the general
shape for "keep trying until the source runs out or a condition is met"
when the source isn't an iterator (over an iterator, a `for` loop is the
idiomatic form — Clippy's `while_let_on_iterator` lint suggests exactly
that rewrite) — the
[Reference's loop expressions](https://doc.rust-lang.org/reference/expressions/loop-expr.html)
page defines exactly this semantics for `while let`.

## Embedded Rust Notes

**Full support.** `while` polling loops are a staple of bare-metal
embedded code (spinning on a status register bit until a peripheral is
ready) when interrupts aren't used instead.
