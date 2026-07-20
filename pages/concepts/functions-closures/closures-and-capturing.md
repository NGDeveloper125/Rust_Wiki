---
title: "Closures & capturing"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures", "Functional Programming"]
related_syntax: ["|...| closures", "->", move, "||"]
see_also: ["Functions", "Fn / FnMut / FnOnce", "Higher-order functions", "Move semantics"]
---

## Explanation

A closure is an anonymous, inline function-like value, written
`|params| body`, that can additionally *capture* variables from the scope
it's defined in — something a plain [function](functions.md) cannot do. A
free `fn` can only see its own parameters and locals; a closure defined
inside a method can reach out and use a variable from that method's body
directly, without it being passed in explicitly.

Capturing exists so that small, situational bits of behavior — "what to
do with this element," "what to run when this event fires," "what to
send on this channel" — can be written right where they're used, carrying
whatever context they need with them. This is what makes closures the
natural argument type for [higher-order functions](higher-order-functions.md)
like iterator adaptors: `.filter(|order| order.total_cents > threshold)`
reads naturally because the closure just reaches out and uses
`threshold` from the surrounding function.

Under the hood, each closure is compiled to an anonymous struct holding
exactly the variables it actually uses, captured the least invasive way
that compiles: by shared reference if the closure only reads a variable,
by mutable reference if it mutates one, or by value if it needs to own
it (or is forced to with the `move` keyword). That generated struct then
implements one or more of `Fn`, `FnMut`, or `FnOnce` depending on how it
uses its captures — see [Fn / FnMut / FnOnce](fn-fnmut-fnonce.md) for what
each one permits.

Capturing by value with `move` is required whenever the closure must
outlive the scope it was written in — most commonly when it's handed to
a spawned thread, since the closure's environment has to keep existing
after the current stack frame returns (see
[Move semantics](move-semantics.md) for why a moved value can no longer
be used from its original binding). A closure that captures nothing at
all needs no environment, and so it coerces automatically to a plain
[function pointer](function-pointers.md).

## Basic usage example

```
let tax_rate = 0.08;
let with_tax = |price: f64| price * (1.0 + tax_rate); // <- closure borrows `tax_rate` from its environment

println!("{}", with_tax(19.99));
```

## Best practices & deeper information

### Scenario: Working with collections

Filtering a list of orders down to the ones above a caller-supplied
threshold is a one-line closure that borrows the threshold from its
enclosing scope, rather than a hand-written loop.

```
struct Order {
    total_cents: u64,
}

fn orders_above(orders: &[Order], threshold_cents: u64) -> Vec<&Order> {
    orders
        .iter()
        .filter(|order| order.total_cents > threshold_cents) // <- closure captures `threshold_cents` by reference
        .collect()
}
```

**Why this way:** an iterator-adaptor closure keeps the filtering
condition next to the data it filters instead of a separate mutable
accumulator loop, the shape the
[Rust Cookbook's collection recipes](https://rust-lang-nursery.github.io/rust-cookbook/data_structures/collections.html)
model throughout.

### Scenario: Multi-threading

A worker thread needs its own copy of its configuration, since it may
still be running after the function that spawned it returns — the
closure passed to `thread::spawn` must `move` that configuration in.

```
use std::thread;

struct WorkerConfig {
    batch_size: usize,
}

fn spawn_worker(config: WorkerConfig) -> thread::JoinHandle<()> {
    thread::spawn(move || { // <- `move` forces the closure to take ownership of `config`
        println!("processing in batches of {}", config.batch_size);
    })
}
```

**Why this way:** `thread::spawn` requires its closure to be `'static`,
since the new thread might outlive the caller's stack frame entirely —
`move` is how the closure's captures stop depending on that frame, per
the [Book's concurrency chapter](https://doc.rust-lang.org/book/ch16-01-threads.html#using-move-closures-with-threads).

### Scenario: Message passing between threads

A background logging thread needs a label that was set up before it
started; capturing that label by `move` keeps it alive inside the
thread's closure for as long as the thread keeps receiving messages.

```
use std::sync::mpsc;
use std::thread;

fn spawn_logger() -> mpsc::Sender<String> {
    let (tx, rx) = mpsc::channel();
    let prefix = String::from("order"); // <- captured by the closure below, not passed as a channel message

    thread::spawn(move || {
        for message in rx {
            println!("[{prefix}] {message}"); // <- `prefix` lives inside the closure's captured environment
        }
    });

    tx
}
```

**Why this way:** the receiving loop's own setup data (here, a constant
label) doesn't need to travel through the channel at all — capturing it
directly in the closure is simpler than threading it through every
message, the pattern the
[Book's message-passing chapter](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
builds its channel examples around.

## Embedded Rust Notes

**Full support.** Closures are a core-language, compile-time construct —
a non-boxed closure passed generically or via `impl Fn`/`impl FnMut`
requires no heap allocation and works identically in `#![no_std]`. Only
*boxing* a closure (`Box<dyn Fn()>`) needs the `alloc` crate plus a
`#[global_allocator]`; capturing itself never does.
