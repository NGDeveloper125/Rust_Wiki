---
title: "if let / while let"
area: "Pattern Matching"
embedded_support: full
groups: ["Pattern Matching"]
related_syntax: ["if let", "while let", "else", match]
see_also: ["match expressions", "Destructuring", "Option<T>", "Result<T, E>"]
---

## Explanation

`if let` and `while let` are shorthand for a [`match`](match-expressions.md)
that only truly cares about one pattern. `if let Some(value) = maybe_value`
runs its block when the pattern matches and does nothing (or runs an
`else` block) when it doesn't, without writing out a `None => {}` arm
that has nothing to say. `while let` is the same idea repeated as a
loop: it keeps running its body for as long as the pattern keeps
matching, and stops the moment it doesn't — which is exactly the shape
of "keep pulling values out of something until it's empty."

The reason these forms exist alongside full `match` is that exhaustive
matching is sometimes the wrong amount of ceremony. A full `match` on
`Option<T>` forces both `Some` and `None` arms to be written even when
the `None` case is "do nothing" — readable once, noisy every time it
recurs across a codebase. `if let` and `while let` say directly "I only
have something to do for this one shape," which is both shorter and
more honest about what the code actually does.

Because they're pattern-matching forms rather than a separate mechanism,
anything you can [destructure](destructuring.md) in a `match` arm you
can destructure in an `if let` or `while let` pattern too — nested
structs, tuple variants, references, all work identically. The
trade-off is exactly the one exhaustiveness buys elsewhere: a `match`
guarantees every case was considered, while `if let` silently ignores
whatever doesn't match, so it's the right tool only when that silence is
genuinely fine (a `let`/`else` binding, or an early-return via `else`, is
often a better fit when the non-matching case actually needs handling).

`while let` most commonly shows up driving a loop off something that
naturally runs dry — draining a `Vec` with `.pop()`, or reading messages
off a channel — where the loop's exit condition *is* the pattern failing
to match, rather than a separately tracked boolean or counter.

## Basic usage example

```
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() { // <- loops for as long as pop() returns Some
    println!("popped {top}");
}

let maybe_name: Option<&str> = Some("Ada");
if let Some(name) = maybe_name { // <- runs only for the Some case, no arm needed for None
    println!("hello, {name}");
}
```

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

A request handler looks up a cached response before doing real work; only
the "found it" case needs code, so `if let` replaces a `match` that would
otherwise carry an empty `None` arm.

```
use std::collections::HashMap;

struct CachedResponse {
    body: String,
}

fn serve(cache: &HashMap<String, CachedResponse>, key: &str) -> String {
    if let Some(cached) = cache.get(key) { // <- only the Some case has anything to do
        return cached.body.clone();
    }
    format!("no cached response for {key}")
}
```

**Why this way:** when only one pattern needs a reaction, `if let` says
so directly instead of making a reader scan a `match` arm that just
returns `()`; the
[Rust Book](https://doc.rust-lang.org/book/ch06-03-concise-control-flow-with-if-let-and-let-else.html)
introduces `if let` specifically as this more concise alternative.

### Scenario: Working with collections

A background worker drains sensor readings from a channel as they
arrive, processing each one until the sending side disconnects and the
channel finally runs dry.

```
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel::<f64>();

thread::spawn(move || {
    for reading in [21.4, 21.6, 21.5] {
        tx.send(reading).unwrap();
    }
}); // tx is dropped here, which is what lets the loop below end

while let Ok(reading) = rx.recv() { // <- loops until recv() returns Err (sender disconnected)
    println!("reading: {reading:.1}");
}
```

**Why this way:** `recv()` returning `Err` is exactly the signal that no
more messages are coming, so `while let Ok(..) = rx.recv()` doubles as
both the extraction and the loop's termination condition — the
[std docs for `mpsc::Receiver::recv`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.recv)
document this disconnect-signals-`Err` behavior directly.

### Scenario: Handling and propagating errors

Saving a configuration file to disk can fail, but the caller only wants
to log the problem and move on rather than propagate it further up —
`if let Err(..)` isolates just that one case.

```
fn save_config(path: &str, contents: &str) -> std::io::Result<()> {
    std::fs::write(path, contents)
}

if let Err(e) = save_config("app.toml", "port = 8080") { // <- only the failure case needs handling here
    eprintln!("failed to save config: {e}");
}
```

**Why this way:** when the success case genuinely needs no follow-up
action, matching only the `Err` arm with `if let` avoids an `Ok(()) => {}`
arm that would say nothing useful — the same "only one shape matters
here" reasoning the
[Rust Book](https://doc.rust-lang.org/book/ch06-03-concise-control-flow-with-if-let-and-let-else.html)
gives for preferring `if let` over a full `match`.

## Embedded Rust Notes

**Full support.** Both forms are core-language, allocator-free, and
compile to the same code a hand-written `match` would. `while let` is a
natural fit for polling a peripheral's status register — `while let
Some(byte) = uart.try_read() { ... }` — draining whatever is available
without a separately tracked count.
