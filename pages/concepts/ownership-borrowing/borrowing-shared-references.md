---
title: "Borrowing (shared references)"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing"]
related_syntax: ["&"]
see_also: ["Ownership", "Mutable borrowing", "The borrow checker"]
---

## Explanation

Borrowing lets code access a value without taking ownership of it. A
shared reference (`&T`) grants read-only access for a limited scope,
after which the original owner remains fully in control — nothing about
ownership changes, and the value is not moved or copied.

This solves a problem ownership alone creates: if passing a value to a
function always moved it, you'd need to pass it back out again (or clone
it) just to keep using it afterward. Borrowing lets a function (or any
other piece of code) *use* a value temporarily without the caller losing
access to it.

Any number of shared references to the same value can exist
simultaneously — this is safe precisely because a shared reference cannot
mutate through it (unless the type uses
[interior mutability](interior-mutability.md); see also
[Immutability by default](immutability-by-default.md)).
The tradeoff for that safety is a lifetime constraint: a reference can
never outlive the value it points to, which the compiler verifies
statically (see [The borrow checker](borrow-checker.md) and
[Lifetimes](lifetimes.md)) rather than checking at runtime the way a
garbage-collected language would.

## Basic usage example

```
let s = String::from("hello");
let r1 = &s;
let r2 = &s; // <- a second shared reference coexists safely with r1

println!("{r1} and {r2}");
println!("{s}"); // s is still usable: borrowing never took ownership
```

**Restriction:** a shared reference only permits reading — mutating
through it (unless the type uses interior mutability), or mutating the
original value while any shared reference to
it is still alive, is rejected at compile time.

## Best practices & deeper information

### Scenario: Sharing data with multiple references

A report function reads two different fields of the same struct through
separate shared borrows at once — safe because neither borrow grants
write access.

```
struct Inventory {
    in_stock: Vec<String>,
    reserved: Vec<String>,
}

fn report(stock: &[String], reserved: &[String]) {
    println!("{} in stock, {} reserved", stock.len(), reserved.len());
}

let inv = Inventory {
    in_stock: vec!["widget".into()],
    reserved: vec!["gadget".into()],
};

report(&inv.in_stock, &inv.reserved); // <- two live shared borrows of `inv`, both read-only
```

**Why this way:** because `&T` never grants write access, any number of
shared borrows of the same or overlapping data can coexist safely — the
[Rust Book](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
calls this out as the reason reading is unrestricted while writing stays
exclusive.

### Scenario: Multi-threading

Several worker threads need read-only access to the same local data
without moving it into each thread or reaching for `Arc` — `thread::scope`
lets them borrow it directly.

```
use std::thread;

let samples = vec![10, 20, 30, 40];

thread::scope(|s| {
    for chunk in samples.chunks(2) {
        s.spawn(move || { // <- borrows `chunk`, itself borrowed from `samples`, never owns it
            let sum: i32 = chunk.iter().sum();
            println!("chunk sum: {sum}");
        });
    }
}); // every scoped thread is joined here; `samples` is still valid afterward
```

**Why this way:** `thread::scope` guarantees every spawned thread finishes
before the scope returns, which is what lets threads borrow local data
directly instead of requiring the `'static` lifetime (and usually an
`Arc`) that plain `thread::spawn` demands — see the
[std docs for `thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html).

## Embedded Rust Notes

**Full support.** Borrowing is a compile-time-only mechanism — no runtime
representation, no allocator, works identically in `#![no_std]`.
