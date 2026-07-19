---
title: "Interior mutability (Cell & RefCell)"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Interior Mutability", "Sharing & Mutating Data Safely"]
related_syntax: []
see_also: ["Mutable borrowing", "The borrow checker", "Shared ownership (Rc & Arc)"]
---

## Explanation

Interior mutability lets you mutate a value through a shared (`&T`)
reference — something the borrow checker otherwise forbids outright — by
moving the exclusivity check from compile time to run time.

`Cell<T>` allows getting and setting a `Copy` value through a shared
reference with no runtime check at all (it never hands out a reference to
the inner value, only whole-value copies in and out, so there's nothing
to check). `RefCell<T>` goes further, handing out actual `&T`/`&mut T`
borrows of its contents on demand, but tracks how many are outstanding at
runtime and panics if the "aliasing XOR mutability" rule would be
violated — the same rule the compiler enforces statically for ordinary
references, just deferred to when the program actually runs.

This exists for the real cases where the compiler's static analysis is
too conservative to accept a genuinely safe pattern: a struct that needs
to update an internal cache from behind a shared reference, or a graph
structure with cyclic references. It's frequently paired with
[`Rc`/`Arc`](shared-ownership-rc-arc.md) (`Rc<RefCell<T>>` is a very
common combination) since shared ownership alone only grants shared
*reference* access — interior mutability is what makes that shared access
mutable too. The cost of this flexibility is that a logic error (two
overlapping mutable borrows of a `RefCell`) becomes a runtime panic
instead of a compile-time error — the safety guarantee is preserved, but
enforcement moves later, and with it the chance of catching the mistake.

## Embedded Rust Notes

**Full support.** Both `Cell` and `RefCell` live in `core::cell` — no
allocator needed. `RefCell`'s runtime borrow tracking is single-threaded,
though, which matters for embedded: it's not safe to share across an
interrupt handler and the main loop the way a `critical-section`-gated
cell or a hardware-mutex-backed type is. Embedded code sharing state with
an interrupt typically reaches for `critical_section::Mutex<RefCell<T>>`
rather than a bare `RefCell<T>`.
