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

## Embedded Rust Notes

**Full support.** Ownership is a compile-time concept enforced regardless
of target — it costs nothing at runtime and requires no allocator, no OS,
and no `std`. If anything, ownership matters *more* in embedded code: a
peripheral, a DMA buffer, or a lock has exactly one owner responsible for
releasing it, with no garbage collector to fall back on if that discipline
slips.
