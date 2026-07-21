---
title: "Shared ownership (Rc & Arc)"
area: "Ownership & Borrowing"
embedded_support: partial
groups: ["Ownership & Borrowing", "Reference Counting", "Sharing & Mutating Data Safely"]
related_syntax: []
see_also: ["Weak references (Weak<T>)", "Interior mutability (Cell & RefCell)", "Ownership"]
---

## Explanation

Ownership's single-owner rule is the right default for most data, but
some structures genuinely need more than one owner at once — a value
shared between several parts of a program where none of them is clearly
"the" owner responsible for cleanup. `Rc<T>` (reference-counted) and
`Arc<T>` (atomically reference-counted, safe to share across threads)
provide this: cloning an `Rc`/`Arc` doesn't deep-copy the inner value, it
increments a count of how many owners currently exist, and the value is
only actually dropped once that count reaches zero.

This effectively moves "how many owners does this have" from a
compile-time-known number (with plain ownership, always exactly one) to a
runtime-tracked count — a controlled, opt-in relaxation of the ownership
model rather than an abandonment of it: the value is still always
eventually dropped deterministically, just at a point determined by
reference count reaching zero rather than a single scope ending — unless
a reference cycle keeps the count from ever reaching zero (see
[Weak references](weak-references.md)).

`Rc<T>`/`Arc<T>` grant only shared (`&T`-style) access to the inner value
by default — they solve "who owns this" but not "how do I mutate it,"
which is why they're so often combined with
[interior mutability](interior-mutability.md) (`Rc<RefCell<T>>`,
`Arc<Mutex<T>>`) when the shared data also needs to change.

## Basic usage example

```
use std::rc::Rc;

let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a); // <- increments the reference count, no deep copy
println!("count = {}", Rc::strong_count(&a));
println!("{a} {b}");
```

## Best practices & deeper information

### Scenario: Shared ownership

A UI-style panel tree needs several panels to read the same theme config,
with no single panel a natural sole owner of it — a job for `Rc`, not for
threading a reference through every constructor.

```
use std::rc::Rc;

struct Theme {
    background: String,
    accent: String,
}

struct Panel {
    theme: Rc<Theme>, // <- shared ownership: several panels reference the same Theme
}

let theme = Rc::new(Theme { background: "#111".into(), accent: "#0af".into() });

let sidebar = Panel { theme: Rc::clone(&theme) }; // <- increments the count, no deep copy
let toolbar = Panel { theme: Rc::clone(&theme) };

println!("live owners: {}", Rc::strong_count(&theme)); // theme + sidebar + toolbar
```

**Why this way:** `Rc` is the right tool exactly when no single struct is
the obvious sole owner of a value several others need to keep alive — the
[Rust Book](https://doc.rust-lang.org/book/ch15-04-rc.html) introduces
`Rc` for graph-like structures like this one, where plain ownership would
force picking one arbitrary owner and threading references everywhere
else.

### Scenario: Multi-threading

The same shared-config pattern, read from multiple OS threads instead of
multiple panels, needs `Arc` rather than `Rc` — `Rc`'s reference count
isn't safe to update from more than one thread.

```
use std::sync::Arc;
use std::thread;

let config = Arc::new(String::from("max_connections=100"));

let handles: Vec<_> = (0..3).map(|i| {
    let config = Arc::clone(&config); // <- cheap, atomic increment; each thread gets its own handle
    thread::spawn(move || {
        println!("worker {i} sees: {config}");
    })
}).collect();

for h in handles {
    h.join().unwrap();
}
```

**Why this way:** `Rc`'s reference count isn't updated atomically, so
cloning it across threads isn't safe (it isn't `Send`); `Arc` uses an
atomic counter to make the exact same shared-ownership pattern
thread-safe, at the cost of slightly more overhead per clone — the
[Rust Book](https://doc.rust-lang.org/book/ch16-03-shared-state.html#atomic-reference-counting-with-arct)
covers this as the reason `Arc` exists alongside `Rc`.

## Explanation (Embedded)

Neither `Rc<T>` nor `Arc<T>` lives in `core` — both are defined in
`alloc`, so using either at all requires `extern crate alloc;` plus a
`#[global_allocator]` configured for the target; on a bare `#![no_std]`
project with no allocator, `Rc::new`/`Arc::new` simply don't compile.
That much is the same "needs `alloc`" caveat as `Box`.

The more honest caveat is about whether reaching for either is the right
move even once `alloc` is available. `Rc`'s whole value proposition —
non-atomic, single-threaded reference counting — only pays for itself
when there's a genuine multi-owner story: several independent parts of a
program, none of which is clearly "the" owner, that outlive individual
function scopes. That shape does happen in embedded code (a shared config
struct read by several long-lived subsystems, for instance), but the
*other* classic reason to reach for `Rc`/`Arc` on a hosted target — "this
needs to survive being handed to a spawned OS thread whose lifetime the
compiler can't otherwise reason about" — usually doesn't apply on bare
metal, because there's rarely an OS thread being spawned in the first
place. What embedded code *does* have instead is an interrupt boundary:
state shared between `main`'s loop and one or more `#[interrupt]`
handlers. For that specific, extremely common case, the idiomatic
solution isn't `Rc`/`Arc` at all — it's a `'static`
`critical_section::Mutex<RefCell<T>>`, which needs no heap, no reference
counting, and no allocator, because the value has exactly one place it
lives (a `static`) and the `Mutex` only governs *when* code is allowed to
touch it, not *how many owners* it has. Reach for `Rc`/`Arc` in embedded
code specifically when the multi-owner shape is real (and `alloc` is
already configured for other reasons) — not as a reflexive substitute for
sharing state with an interrupt handler.

## Basic usage example (Embedded)

```
use core::cell::RefCell;
use critical_section::Mutex;

// The idiomatic embedded default for state shared across the interrupt
// boundary: no alloc, no reference counting, one static owner.
static CONFIG: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(9600)); // <- 'static value, not an Rc/Arc

fn set_baud(new_baud: u32) {
    critical_section::with(|cs| {
        *CONFIG.borrow_ref_mut(cs) = new_baud;
    });
}
```

## Best practices & deeper information (Embedded)

### Scenario: Shared ownership

A baud-rate setting read by the main loop and updated by a configuration
interrupt handler looks like a job for shared ownership, but it's a
`'static` value with two access points, not several genuine owners — the
`alloc`-free answer is a `critical_section::Mutex`, not `Rc`.

```
use core::cell::RefCell;
use critical_section::Mutex;

static BAUD_RATE: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(9600)); // <- one static owner, not a reference count

#[interrupt]
fn CONFIG_UPDATE() {
    critical_section::with(|cs| {
        *BAUD_RATE.borrow_ref_mut(cs) = 115_200;
    });
}

fn main_loop_read() -> u32 {
    critical_section::with(|cs| *BAUD_RATE.borrow(cs))
}
```

**Why this way:** `main` and `CONFIG_UPDATE` aren't two independent
*owners* of the baud rate the way two threads holding `Arc` clones would
be — there's exactly one value, living in one `static`, and the two
contexts just need mutually-exclusive turns touching it, which is what
`critical-section`'s `Mutex` provides without a heap; reaching for `Rc`
here would add an allocation and a reference count to solve a problem
that's actually about mutual exclusion, not ownership count, per the
[Rust Book's framing of `Rc` as "multiple ownership"](https://doc.rust-lang.org/book/ch15-04-rc.html)
specifically — which this scenario doesn't have.

### Scenario: Multi-threading

On the (less common) embedded targets that run a real preemptive RTOS
with `alloc` configured and OS-level threads — FreeRTOS via
`freertos-rust`, or ESP-IDF's `std` support on Xtensa/RISC-V — `Arc` earns
its keep exactly the way it does on a hosted target, because the
OS-thread-based use case it was built for genuinely exists there.

```
// Illustrative: an RTOS/std-capable embedded target (e.g. ESP-IDF with std support)
use std::sync::{Arc, Mutex};
use std::thread;

let shared_reading = Arc::new(Mutex::new(0.0_f32));

let worker = {
    let shared_reading = Arc::clone(&shared_reading); // <- genuine second owner: a real OS thread
    thread::spawn(move || {
        *shared_reading.lock().unwrap() = 21.4;
    })
};

worker.join().unwrap();
println!("{}", *shared_reading.lock().unwrap());
```

**Why this way:** this only applies on the subset of embedded targets
that actually provide OS threads and an allocator (an RTOS port or a
`std`-capable target) — on a bare `#![no_std]` project with no RTOS, this
use case doesn't arise, and the interrupt-boundary scenario above is the
one to reach for instead; where a real OS thread does exist, `Arc`'s
atomic reference count is exactly as necessary as it is on a desktop
target, per the same
[Rust Book reasoning for `Arc` over `Rc`](https://doc.rust-lang.org/book/ch16-03-shared-state.html#atomic-reference-counting-with-arct).
