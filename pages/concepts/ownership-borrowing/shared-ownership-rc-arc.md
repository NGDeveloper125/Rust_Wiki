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
reference count reaching zero rather than a single scope ending.

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

## Embedded Rust Notes

**Partial support.** Neither `Rc` nor `Arc` is in `core` — both live in
`alloc`, so they require `extern crate alloc;` plus a `#[global_allocator]`
configured for your target. Many embedded projects deliberately avoid a
heap allocator altogether (to get fully deterministic, bounded memory use)
and reach for `heapless` fixed-capacity structures or static
(`'static`) references shared via borrowing instead of runtime reference
counting. When a project *does* set up a global allocator (common on
larger microcontrollers running an RTOS), `Rc`/`Arc` work exactly as on a
hosted target.
