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
mutate through it (see [Immutability by default](immutability-by-default.md)).
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
through it, or mutating the original value while any shared reference to
it is still alive, is rejected at compile time.

## Embedded Rust Notes

**Full support.** Borrowing is a compile-time-only mechanism — no runtime
representation, no allocator, works identically in `#![no_std]`.
