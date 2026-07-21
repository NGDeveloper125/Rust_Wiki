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
`&mut self` signals "mutates in place without taking ownership" — pick
the receiver that matches what the method actually does, since callers
rely on it to know whether they keep the value; the
[Rust Book](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)
covers the `self`/`&self`/`&mut self` semantics behind each choice.

## Explanation (Embedded)

Ownership costs nothing at runtime and needs no allocator, OS, or `std` —
it's a compile-time concept, so everything in the classic Explanation
holds without modification under `#![no_std]`. Embedded Rust also has one
of the strongest, most concrete illustrations of *why* single ownership
matters: the **singleton peripheral pattern**, which nearly every
embedded-hal-based crate (the STM32, RP2040, nRF52, and SAMD HAL families
among them) uses to model hardware.

A microcontroller's registers are a fixed, singular physical resource —
there is exactly one UART1, one GPIOA bank, one TIM2 timer on the chip.
The peripheral-access crate (PAC) generated for that chip exposes a single
`Peripherals` struct whose fields are handles to each of these blocks, and
a `Peripherals::take()` function that hands the whole struct out **at
most once**, returning `None` on every call after the first. Doing this
turns ownership into a hardware-safety guarantee: if you have a `Gpioa`
value, you know statically that no other code in the program can also
have one, so no other code can be reconfiguring the same GPIO bank's mode
register while you're mid-configuration. This is exactly the same
"exactly one owner, enforced at compile time" idea the classic Explanation
describes for a `String` — applied here to a piece of physical hardware
instead of a heap allocation, which is arguably a more intuitive place for
it to click: a microcontroller pin genuinely does have only one wire, so
letting exactly one owner exist for it isn't a compiler restriction, it's
a fact about the hardware that the type system is finally able to state
directly.

Once obtained, that ownership composes exactly like the classic
Explanation describes: passing `gpioa` into a driver's constructor moves
it again, so the driver becomes the pin's new sole owner, and the caller
loses the ability to touch it directly — see
[Move semantics](move-semantics.md) for that half of the picture.

## Basic usage example (Embedded)

```
struct Gpioa; // stand-in for a PAC-generated peripheral type

struct Peripherals { gpioa: Gpioa }

static TAKEN: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

impl Peripherals {
    fn take() -> Option<Self> {
        if TAKEN.swap(true, core::sync::atomic::Ordering::AcqRel) {
            None // <- already taken once: no second owner of the hardware is possible
        } else {
            Some(Peripherals { gpioa: Gpioa })
        }
    }
}

let p1 = Peripherals::take(); // Some(...): first call succeeds
let p2 = Peripherals::take(); // None: the hardware is already owned
assert!(p1.is_some());
assert!(p2.is_none());
```

## Best practices & deeper information (Embedded)

### Scenario: Transferring ownership

Taking the chip's `Peripherals` once and moving individual peripherals
into the drivers that use them chains single ownership all the way from
"raw hardware" to "the one driver responsible for it."

```
struct Gpioa; // stand-in for a PAC-generated peripheral type
struct Peripherals { gpioa: Gpioa }
struct Led { pin: Gpioa }

impl Peripherals {
    fn take() -> Option<Self> { Some(Peripherals { gpioa: Gpioa }) } // simplified for this example
}

impl Led {
    fn new(pin: Gpioa) -> Self { // <- ownership moves again: the Led is now the pin's sole owner
        Led { pin }
    }
}

let peripherals = Peripherals::take().unwrap();
let led = Led::new(peripherals.gpioa); // <- peripherals.gpioa moves into led; it can't be reused from `peripherals`
```

**Why this way:** each move narrows who is responsible for the hardware —
first the whole-chip `Peripherals` struct, then one specific bank, then
the driver that configures it — so at every point in the program exactly
one piece of code can legally touch the pin, which is the same "handing
off responsibility entirely" idea the
[Rust Book](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
describes for ordinary values, just applied down a chain of constructors
instead of into one collection.

### Scenario: Designing a public API

A PAC's `Peripherals::take()` function has to guard against being called
twice — including from two different execution contexts, such as `main`
and an interrupt handler — so its implementation needs to be atomic, not
just "check a boolean."

```
use core::sync::atomic::{AtomicBool, Ordering};

struct Gpioa; // stand-in for a PAC-generated peripheral type

pub struct Peripherals {
    pub gpioa: Gpioa,
}

static TAKEN: AtomicBool = AtomicBool::new(false);

impl Peripherals {
    pub fn take() -> Option<Self> {
        // <- swap is a single atomic operation: two simultaneous callers can't both see `false`
        if TAKEN.swap(true, Ordering::AcqRel) {
            None
        } else {
            Some(Peripherals { gpioa: Gpioa })
        }
    }
}
```

**Why this way:** returning `Option<Self>` from a plain, safe function —
rather than requiring callers to write `unsafe` or trust a convention —
puts the "only one owner" guarantee in the type system itself, where the
compiler enforces it; using `AtomicBool::swap` instead of a
separate load-then-store keeps the check race-free even if `take()` is
somehow called from both `main` and an interrupt, which the
[API Guidelines](https://rust-lang.github.io/api-guidelines/predictability.html)'
predictability guidance favors over an API that merely documents "call
this only once" and hopes.
