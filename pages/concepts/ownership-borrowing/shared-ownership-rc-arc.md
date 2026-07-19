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
