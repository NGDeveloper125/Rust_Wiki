---
title: "Fn / FnMut / FnOnce"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures", "Functional Programming"]
related_syntax: ["|", move, fn]
see_also: ["Closures & capturing", "Higher-order functions", "Trait bounds"]
---

## Explanation

`Fn`, `FnMut`, and `FnOnce` are the three traits that describe *how* a
closure is allowed to use whatever it captured — not what a closure
looks like syntactically, but what calling it is allowed to do. `Fn`
means the closure can be called through a shared reference, any number of
times (it only reads its captures). `FnMut` means it needs exclusive
access to call it (it mutates a capture), but can still be called
repeatedly. `FnOnce` means calling it consumes it — it may only be called
once, because doing so moves out of (or drops) something it captured by
value. Every [closure](closures-and-capturing.md) automatically implements
whichever of these its body's actual behavior allows; nobody writes
`impl Fn for ...` by hand for a closure literal.

The three form a hierarchy, not three unrelated options: every `Fn`
closure is also a valid `FnMut`, and every `FnMut` closure is also a
valid `FnOnce` — calling something zero-or-more times without mutation is
also fine to do just once with mutation allowed, and so on. This is
expressed as a [supertrait](../traits-polymorphism/supertraits.md)
relationship (`Fn: FnMut: FnOnce`), which is why a function that only
requires `FnOnce` accepts a strictly wider range of caller-supplied
closures than one that requires `Fn`.

This hierarchy is exactly why choosing the bound matters when
[writing a function that accepts a closure](higher-order-functions.md):
requiring the loosest trait the function actually needs — `FnOnce` for a
single consuming call, `FnMut` when the closure needs to mutate something
it captured across repeated calls, `Fn` only when it must be callable
through a shared reference — keeps the function usable by the widest set
of closures a caller might reasonably want to pass in.

A closure's *capture mode* (by reference, by mutable reference, or by
value/`move`) is what actually determines the strictest of the three
traits a given closure literal satisfies, but that's a separate concern
from which trait a function's signature *requires* — see
[Closures & capturing](closures-and-capturing.md) for the capture side of
this. A plain [function pointer](function-pointers.md) captures nothing at
all, so it always implements all three automatically.

## Basic usage example

```
fn apply_twice<F: Fn(i32) -> i32>(value: i32, f: F) -> i32 { // <- `Fn`: `f` may be called more than once here
    f(f(value))
}

let doubled_twice = apply_twice(5, |n| n * 2);
```

## Best practices & deeper information

### Scenario: Writing generic code

A retry helper calls its closure repeatedly and needs it to mutate a
captured attempt counter between calls, which rules out `Fn` and requires
`FnMut`.

```
fn retry_until_success<F: FnMut() -> bool>(mut attempt: F, max_tries: u32) -> bool {
    // <- `FnMut`: called repeatedly, and needs `&mut` access to its captures
    for _ in 0..max_tries {
        if attempt() {
            return true;
        }
    }
    false
}

let mut tries_used = 0;
let succeeded = retry_until_success(
    || {
        tries_used += 1; // <- mutates a captured variable, so this closure only satisfies `FnMut`, not `Fn`
        tries_used >= 3
    },
    5,
);
```

**Why this way:** the
[Book's closures chapter](https://doc.rust-lang.org/book/ch13-01-closures.html)
frames the choice between `Fn`, `FnMut`, and `FnOnce` as matching the
bound to what the closure body actually does with its captures, rather
than defaulting to the most permissive-looking one.

### Scenario: Multi-threading

`thread::spawn` calls its closure exactly once, on the new thread, so it
only requires `FnOnce` — a stricter bound like `Fn` would needlessly
reject closures that consume what they captured.

```
use std::thread;

fn run_in_background<F: FnOnce() + Send + 'static>(job: F) {
    // <- `FnOnce`: `thread::spawn` calls the closure exactly one time
    thread::spawn(job);
}

let payload = String::from("order-42");
run_in_background(move || {
    println!("processing {payload}"); // <- consumes `payload`, which is why only `FnOnce` is required
});
```

**Why this way:** the
[standard library's `thread::spawn` signature](https://doc.rust-lang.org/std/thread/fn.spawn.html)
bounds its closure parameter by `FnOnce() -> T + Send + 'static` for
exactly this reason — a thread's entry point runs once and is free to
consume its captures.

### Scenario: Designing a public API

An event-driven `Button` needs to run its click handler on every click,
and the handler often needs to mutate something it captured (a counter,
a log) — so the field's type is bounded by `FnMut`, not `Fn` or `FnOnce`.

```
struct Button {
    on_click: Box<dyn FnMut()>, // <- `FnMut`: may run on every click, and may mutate captured state each time
}

impl Button {
    fn click(&mut self) {
        (self.on_click)();
    }
}

let mut click_count = 0;
let mut button = Button {
    on_click: Box::new(move || {
        click_count += 1;
        println!("clicked {click_count} times");
    }),
};
button.click();
button.click();
```

**Why this way:** requiring the least restrictive trait a callback API
actually needs admits the widest range of caller closures, since every
`Fn` closure also satisfies `FnMut` — an argument
[Effective Rust](https://effective-rust.com/) makes when discussing how to
choose between the three closure traits in a public signature.

## Embedded Rust Notes

**Full support.** `Fn`, `FnMut`, and `FnOnce` live in `core::ops`, not
`std`, so bounding a generic parameter or using `impl Fn`/`impl FnMut` by
value works identically in `#![no_std]` with no runtime cost. Only
storing one as a trait object (`Box<dyn FnMut()>`) needs the `alloc`
crate; the traits themselves never do.
