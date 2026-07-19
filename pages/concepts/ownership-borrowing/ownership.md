---
title: "Ownership"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Unique to Rust", "Coming from Python / JavaScript", "Coming from Java / C#", "Coming from C / C++"]
related_syntax: [let, mut, move]
see_also: ["The borrow checker", "Move semantics", "Borrowing (shared references)"]
---

## Explanation

Every value in Rust has exactly one owner at any given time — the
variable, struct field, or collection slot currently responsible for it.
When the owner goes out of scope, the value is dropped and its memory (or
any other resource it holds — a file handle, a lock, a socket) is
released automatically, deterministically, at that exact point.

This single rule is what lets Rust manage memory without a garbage
collector and without requiring the programmer to call `free`/`delete`
manually. There is no ambiguity about who's responsible for cleanup,
because there is never more than one owner: if you pass a value to a
function, assign it to another variable, or put it in a collection,
ownership *moves* — the original binding stops being usable, and the new
location is now solely responsible for the value (see
[Move semantics](move-semantics.md)).

Ownership is Rust's foundational idea — nearly everything else in the
language (borrowing, lifetimes, `Drop`, `Rc`/`Arc` for the cases where a
single owner genuinely isn't enough) exists either to work within this
rule or to provide a controlled, explicit way to relax it. Understanding
ownership first is what makes the rest of the ownership-and-borrowing
system click, rather than feeling like a wall of arbitrary compiler
complaints.

## Basic usage example

```
let s1 = String::from("hello");
let s2 = s1; // <- ownership of the String moves from s1 to s2 here

println!("{s2}"); // fine: s2 owns the value now
// println!("{s1}"); // would fail to compile: s1 no longer owns anything
```

**Restriction:** once ownership moves, the old binding (`s1`) can no
longer be used — this is enforced at compile time, not left as a runtime
footgun.

## Best practices & deeper information

### Scenario: Transferring ownership

A function that hands an order off to a queue should take ownership
outright, rather than borrowing it — the queue becomes the new home for
the value, and the caller genuinely shouldn't keep using it afterward.

```
struct Order {
    id: u64,
    total_cents: u64,
}

fn enqueue(order: Order, queue: &mut Vec<Order>) {
    queue.push(order); // <- ownership of `order` moves into the Vec here
}

let order = Order { id: 42, total_cents: 1999 };
let mut pending = Vec::new();
enqueue(order, &mut pending);
// order.id would fail to compile here: order was moved into enqueue
```

**Why this way:** passing ownership instead of a reference is the right
call whenever the callee genuinely becomes the value's new home (a queue,
a collection, a spawned thread) — the
[Rust Book](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
frames a move as handing off responsibility for the value entirely, which
is exactly what enqueueing means here.

### Scenario: Serving a web endpoint

An axum handler needs read access to shared application state on every
request, but no single request can become that state's owner — other
concurrent requests need it too.

```
// [dependencies] axum = "0.8", tokio = { version = "1", features = ["full"] }
use axum::{extract::State, routing::get, Json, Router};
use std::sync::{atomic::{AtomicU64, Ordering}, Arc};

struct AppState {
    order_count: AtomicU64,
}

async fn order_count(State(state): State<Arc<AppState>>) -> Json<u64> {
    // <- `state` is a shared, cloned Arc handle, never an owned AppState
    Json(state.order_count.load(Ordering::Relaxed))
}

fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/orders/count", get(order_count))
        .with_state(state) // <- each request gets a cheap Arc clone, never ownership of the original
}
```

**Why this way:** handler state must outlive any single request and stay
reachable from many concurrent requests at once, which rules out any one
handler truly owning it — axum's state extraction is built around cloning
an `Arc` per request rather than moving the state in, per the
[axum docs](https://docs.rs/axum/latest/axum/extract/struct.State.html).

### Scenario: Designing a public API

Choosing a method's receiver — owned `self`, `&self`, or `&mut self` — is
part of the API's contract, not an implementation detail: it tells the
caller exactly what happens to their ownership when they call it.

```
struct RequestBuilder {
    url: String,
    retries: u32,
}

impl RequestBuilder {
    fn retries(mut self, n: u32) -> Self { // <- owned self: enables method chaining, consumes the builder
        self.retries = n;
        self
    }

    fn url(&self) -> &str { // <- &self: read-only inspection, caller keeps ownership
        &self.url
    }

    fn reset_retries(&mut self) { // <- &mut self: in-place mutation, no ownership change
        self.retries = 0;
    }
}
```

**Why this way:** owned `self` signals "this call consumes and reshapes
the value" (ideal for chained builders), `&self` signals "read-only," and
`&mut self` signals "mutates in place without taking ownership" — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/predictability.html)
recommend picking the receiver that matches what the method actually
does, since callers rely on it to know whether they keep the value.

## Embedded Rust Notes

**Full support.** Ownership is a compile-time concept enforced regardless
of target — it costs nothing at runtime and requires no allocator, no OS,
and no `std`. If anything, ownership matters *more* in embedded code: a
peripheral, a DMA buffer, or a lock has exactly one owner responsible for
releasing it, with no garbage collector to fall back on if that discipline
slips.
