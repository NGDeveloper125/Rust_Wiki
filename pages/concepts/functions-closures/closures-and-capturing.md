---
title: "Closures & capturing"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures", "Functional Programming"]
related_syntax: ["|", "->", move, "||"]
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
[Move semantics](../ownership-borrowing/move-semantics.md) for why a moved value can no longer
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

## Explanation (Embedded)

Closures are a compile-time, core-language construct, so everything about
how a closure is compiled — the anonymous per-closure struct, capture-by-
reference/mutable-reference/value — works exactly the same inside
`#![no_std]`. A closure passed generically (`impl Fn`/`impl FnMut`) or by
reference (`&mut dyn FnMut`) needs no heap allocation at all: its captured
environment lives on the stack (or is embedded directly in the calling
frame), just like on a hosted target.

The one genuinely embedded-relevant pattern this unlocks is registering a
closure as a callback into a HAL driver or interrupt handler while it
still holds `&mut` access to a peripheral — for example, a GPIO driver's
"on interrupt, call this" slot taking `impl FnMut()`, where the closure
captures `&mut` a shared counter or a second peripheral it needs to touch
each time the interrupt fires. Because the closure is generic (or passed
by reference), this costs nothing beyond the call itself: no vtable, no
heap.

The one caveat worth being explicit about: that's only true for a
non-owning/non-boxed closure. A closure captured by `move` and then boxed
into a `Box<dyn FnMut()>` — the shape you'd reach for to store a
heterogeneous collection of callbacks, or to return a closure without
naming its concrete type — needs the `alloc` crate plus a
`#[global_allocator]`, exactly as on a hosted target. If a callback only
ever needs to be called through a generic bound or a plain reference,
skip boxing it: `impl FnMut()` or `&mut dyn FnMut()` never touches the
heap.

## Basic usage example (Embedded)

```
struct Gpio {
    state: bool,
}

impl Gpio {
    fn toggle(&mut self) {
        self.state = !self.state;
    }
}

fn on_interrupt(mut handler: impl FnMut()) { // <- generic bound: no heap allocation, no vtable
    handler();
}

fn demo(led: &mut Gpio) {
    on_interrupt(|| led.toggle()); // <- closure captures `led` by mutable reference
}
```

## Best practices & deeper information (Embedded)

### Scenario: Mutating through a reference

A GPIO interrupt fires repeatedly for as long as the pin is armed, and the
handler needs exclusive access to toggle the pin each time — so the
callback slot is typed to accept a closure that captures `&mut` access to
the peripheral, not an owned copy of it.

```
struct Led {
    on: bool,
}

impl Led {
    fn toggle(&mut self) {
        self.on = !self.on;
    }
}

fn register_gpio_callback<F: FnMut()>(mut callback: F) {
    // <- `FnMut`: the interrupt may fire many times, and the closure mutates its capture each time
    for _ in 0..3 {
        callback(); // simulates repeated interrupt firings
    }
}

fn wire_up(led: &mut Led) {
    register_gpio_callback(|| led.toggle()); // <- closure captures `led` by `&mut`, not by value
}
```

**Why this way:** capturing the peripheral by mutable reference instead
of moving it in keeps the peripheral usable by the rest of `wire_up`
after registration, and avoids needing `'static` or ownership transfer
just to hand a callback to an interrupt-registration function.

### Scenario: Designing a public API

A sensor driver's "new data" callback needs to run on every sample and
often needs to mutate something it captured (a running total, a flag) —
so the driver's API takes `impl FnMut(u16)` generically rather than a
boxed trait object, keeping the whole call chain heap-free.

```
struct Driver;

impl Driver {
    fn on_data_ready<F: FnMut(u16)>(&mut self, mut callback: F) {
        // <- generic `FnMut` parameter: works with a plain reference-capturing closure, no `alloc` needed
        let sample: u16 = 512; // stand-in for a register read
        callback(sample);
    }
}

fn read_and_log(driver: &mut Driver, total: &mut u32) {
    driver.on_data_ready(|sample| *total += u32::from(sample)); // <- captures `total` by `&mut`
}
```

**Why this way:** a generic `impl FnMut` parameter is monomorphized per
call site and never allocates, whereas storing the same callback as
`Box<dyn FnMut(u16)>` would require the `alloc` crate and a global
allocator — reach for the boxed form only when the driver genuinely needs
to store a heterogeneous collection of callbacks, not just call one back
synchronously.
