---
title: "move"
kind: keyword
embedded_support: full
groups: ["Ownership & Borrowing", "Functions & Closures", "Concurrency & Async"]
related_concepts: ["Move semantics", "Closures & capturing"]
related_syntax: ["|", "async"]
see_also: ["async", "|"]
---

## Explanation

`move` appears immediately before a closure's parameter list (`move |x| ...`)
or immediately after `async` and before an async block or async closure
(`async move { ... }`, `async move |x| ...`). In both positions it changes
how *every* variable the closure or block uses from its surrounding scope is
captured: instead of the compiler picking the least invasive mode per
variable (shared reference if only read, mutable reference if mutated, and
only by value if the body forces it), `move` forces **all** of them to be
taken by value, unconditionally. There is no partial form — you cannot write
`move` and have it apply to some captured variables but not others; if you
need a mix, the usual technique is to pre-bind the variables you want moved
under shadowed names (often clones of shared handles) just before the
closure, so the closure's own captures are uniformly by value.

`move` has no effect on variables the closure doesn't actually use — the
compiler still only captures what the body references, `move` just changes
*how* those captures happen, not *which* ones exist. It also has no bearing
on which of `Fn`/`FnMut`/`FnOnce` the resulting closure implements; that's
determined separately by what the body does with its captures (reads only,
mutates, or consumes). A `move` closure that only reads its by-value
captures still implements `Fn`.

A subtlety worth knowing at the syntax level: for a `Copy` type, "moving" a
capture just copies it — the original binding outside the closure remains
valid and usable afterward, because nothing was invalidated, only
duplicated. For a non-`Copy` type, the original binding is invalidated the
moment the closure literal is created, exactly as any other move would
invalidate it (see [Move semantics](../../concepts/ownership-borrowing/move-semantics.md)
for why that's true of moves generally, and
[Closures & capturing](../../concepts/functions-closures/closures-and-capturing.md)
for when forcing capture-by-value is the right call in the first place).

`move` is not legal on a plain `fn` — free functions and associated
functions have no environment to capture from at all, so there is nothing
for `move` to apply to. It is exclusively a closure/async-block modifier.

## Basic usage example

```
let price = 19.99;
let compute_total = move |quantity: u32| price * quantity as f64; // <- forces `price` to be captured by value

println!("{}", compute_total(3));
println!("{price}"); // still valid: f64 is Copy, so `move` copied it rather than invalidating `price`
```

## Best practices & deeper information

### Scenario: Multi-threading

A spawned thread's closure has to own everything it touches, since the
thread may still be running after the function that spawned it returns —
`move` sits directly before the closure's `||`.

```
use std::thread;

let sensor_ids = vec![101, 102, 103];

let handle = thread::spawn(move || {
    // <- `move` forces every capture (`sensor_ids`) to be taken by value, not borrowed
    let total: u32 = sensor_ids.iter().sum();
    println!("total: {total}");
});

handle.join().unwrap();
```

**Why this way:** `thread::spawn` requires its closure to be `'static`
because the new thread isn't bounded by the caller's stack frame, so nothing
it captures may borrow from that frame — the
[Book's concurrency chapter](https://doc.rust-lang.org/book/ch16-01-threads.html#using-move-closures-with-threads)
covers `move` as the standard way to satisfy that bound.

### Scenario: Message passing between threads

A producer closure needs to own both the channel's `Sender` and the data it
sends, so that the `Sender` is dropped (and the channel closed) exactly when
the closure finishes, independent of anything in the caller's scope.

```
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel::<String>();
let orders = vec!["order-1".to_string(), "order-2".to_string()];

thread::spawn(move || {
    // <- `move` takes ownership of both `tx` and `orders`, not just one of them
    for order in orders {
        tx.send(order).unwrap();
    }
}); // the closure's `tx` is dropped when it returns, which is what closes the channel

for received in rx {
    println!("got: {received}");
}
```

**Why this way:** dropping every clone of the `Sender` is what signals the
receiving end that no more messages are coming — the
[Book's message-passing chapter](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
relies on `move` to put the `Sender` inside the closure so it drops with it,
rather than lingering in the spawning function's scope.

### Scenario: Async tasks

A task handed to an async runtime can outlive the function that spawned it,
just like a thread — `move` follows `async` for exactly the same reason it
precedes `||` on a thread closure.

```
// [dependencies] tokio = { version = "1", features = ["full"] }
use tokio::task;

async fn process_batch(order_ids: Vec<u32>) {
    let handle = task::spawn(async move {
        // <- `move` follows `async`, forcing `order_ids` into the future's captured state
        let total: u32 = order_ids.iter().sum();
        println!("batch total: {total}");
    });

    handle.await.unwrap();
}
```

**Why this way:** `tokio::spawn` requires its future to be `'static` for the
same reason `thread::spawn` does — the
[Tokio tutorial](https://tokio.rs/tokio/tutorial/spawning) uses `async move`
whenever a spawned task needs to own data from its surrounding function
rather than borrow it.

## Embedded Rust Notes

**Full support.** `move` is core-language and allocator-free — capturing by
value costs nothing extra by itself (only *boxing* a closure, e.g.
`Box<dyn FnOnce()>`, needs `alloc`). `move` closures are exactly as usable in
`#![no_std]` code (e.g. handing owned state to a `critical-section` closure
or an RTIC task) as in hosted Rust.
