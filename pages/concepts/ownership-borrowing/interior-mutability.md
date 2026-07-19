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

## Basic usage example

```
use std::cell::RefCell;

let data = RefCell::new(5);
*data.borrow_mut() += 1; // <- mutates through a shared RefCell value
println!("{}", data.borrow());
```

**Restriction:** `RefCell`'s borrow rules are checked at runtime, not
compile time — holding two overlapping `borrow_mut()`s (or a `borrow()`
alongside a `borrow_mut()`) panics instead of failing to compile.

## Best practices & deeper information

### Scenario: Modifying an existing object

A single-threaded lookup type caches its last computed result behind a
`RefCell`, so `&self` methods can still update the cache internally
without forcing every caller through `&mut self`.

```
use std::cell::RefCell;

struct PriceLookup {
    cache: RefCell<Option<(String, f64)>>,
}

impl PriceLookup {
    fn price_for(&self, sku: &str) -> f64 { // <- `&self`, not `&mut self`: mutation stays internal
        if let Some((cached_sku, price)) = &*self.cache.borrow() {
            if cached_sku == sku {
                return *price;
            }
        }
        let price = 19.99; // pretend this is an expensive lookup
        *self.cache.borrow_mut() = Some((sku.to_string(), price)); // <- mutates through &self
        price
    }
}
```

**Why this way:** `RefCell` lets a type present a read-only (`&self`)
public API while still updating internal bookkeeping like a cache — the
[Rust Book](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
frames this as the case interior mutability exists for: a mutation that's
an implementation detail, not part of the type's externally visible
contract.

### Scenario: Sharing state across threads

The moment that single-threaded cache needs to be touched from more than
one thread, `RefCell` stops being an option — the fix is `Mutex`, not a
thread-safe variant of `RefCell`.

```
use std::sync::Mutex;

struct SharedCounter {
    hits: Mutex<u64>, // <- Mutex, not RefCell: RefCell's borrow tracking isn't thread-safe
}

impl SharedCounter {
    fn record_hit(&self) {
        let mut hits = self.hits.lock().unwrap(); // <- blocks other threads instead of racing
        *hits += 1;
    }
}
```

**Why this way:** `RefCell` doesn't implement `Sync`, so it can't be
shared across threads at all — its runtime borrow check has no protection
against two threads racing on it simultaneously; `Mutex` is the
thread-safe equivalent, enforcing exclusivity by blocking instead of
panicking, as the
[Rust Book](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
covers when introducing `Mutex<T>`.

## Embedded Rust Notes

**Full support.** Both `Cell` and `RefCell` live in `core::cell` — no
allocator needed. `RefCell`'s runtime borrow tracking is single-threaded,
though, which matters for embedded: it's not safe to share across an
interrupt handler and the main loop the way a `critical-section`-gated
cell or a hardware-mutex-backed type is. Embedded code sharing state with
an interrupt typically reaches for `critical_section::Mutex<RefCell<T>>`
rather than a bare `RefCell<T>`.
