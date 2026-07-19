---
title: "Immutability by default"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Functional Programming", "Coming from Python / JavaScript"]
related_syntax: [let, mut]
see_also: ["Ownership", "Mutable borrowing"]
---

## Explanation

Every binding introduced with `let` is immutable unless explicitly marked
`mut`. This is the opposite default from most mainstream languages (Java,
Python, JavaScript, C), where variables are mutable unless specially
declared `const`/`final`.

The practical effect is that immutability becomes the norm you reach for,
and mutability becomes something you opt into deliberately at each
binding site — a small, local signal to a reader that *this* particular
variable is expected to change, which makes the ones that don't stand out
as safe to reason about without tracking their value over time.

This default also interacts with the borrow checker directly: a shared
reference (`&T`) can never be used to mutate through it, precisely
because immutability-by-default is the baseline the whole borrowing model
is built on top of — mutability is the special case that needs an
explicit `&mut` to unlock, not the other way around. This is a large part
of why data races are ruled out at compile time: you cannot have two
simultaneous mutable accesses to the same data without the compiler
seeing an explicit `&mut` for it.

## Embedded Rust Notes

**Full support.** No `std`/allocator dependency — the immutable-by-default
rule is enforced identically on every target.
