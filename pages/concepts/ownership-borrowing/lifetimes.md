---
title: "Lifetimes"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Lifetime Management", "Unique to Rust"]
related_syntax: ["'ident", "&"]
see_also: ["The borrow checker", "Lifetime elision", "Borrowing (shared references)"]
---

## Explanation

A lifetime describes how long a reference remains valid — specifically,
that it cannot outlive the value it points to. Every reference in Rust
has a lifetime, whether or not it's written out explicitly; naming it
(`'a`) becomes necessary only when the compiler can't work out on its own
that a function's inputs and outputs relate correctly.

The concept lifetimes exist to express is straightforward: `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str`
says "the reference this function returns lives no longer than the
shorter-lived of the two inputs" — a real constraint on the function's
contract, not a formality. Without that annotation, the compiler would
have no way to verify the caller isn't left holding a dangling reference
after the inputs it borrowed from go out of scope.

Lifetimes are a **compile-time-only** concept — they exist purely to let
the borrow checker prove reference validity ahead of time, and are erased
entirely before the program runs (there is no runtime lifetime tracking,
no reference counting implied by writing `'a`). This is precisely why
Rust can guarantee no dangling references with zero runtime cost: the
proof happens once, at compile time, instead of via a runtime check
(garbage collection, reference counting) every single language with
manual memory management otherwise needs.

## Embedded Rust Notes

**Full support.** Lifetimes are erased entirely before codegen — a purely
compile-time concept with zero runtime footprint, identical on every
target including `#![no_std]`.
