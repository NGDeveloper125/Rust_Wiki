---
title: "mut"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Ownership, Borrowing (shared references), Mutable borrowing]
related_syntax: [let, "&"]
see_also: [let, "&"]
---

## Explanation

`mut` marks a binding, reference, or raw pointer as mutable — the one
thing in Rust that is *not* mutable by default. It appears in a few
distinct syntactic positions that are easy to conflate:

- **On a `let` binding:** `let mut x = 5;` allows `x` itself to be
  reassigned or mutated later. Without `mut`, `x = 6;` is a compile error.
- **On a reference type:** `&mut T` is a mutable (exclusive) reference —
  a different type from `&T`, not a modifier of it. Only one `&mut T` to a
  given value can exist at a time, and it cannot coexist with any `&T`.
- **On a function parameter pattern:** `fn f(mut x: i32)` makes the
  parameter binding mutable inside the function body — this is purely
  local; it says nothing about the caller's variable and has no effect on
  the function's signature/type.
- **On `self`:** `&mut self` in a method signature borrows the receiver
  mutably.

`mut` is not part of a type in the `let mut x` sense (the binding is
mutable, not the type `i32`), but it *is* part of the type in the
reference sense (`&mut T` and `&T` are different types entirely).

## Basic usage example

```
let mut x = 5; // <- `mut` allows `x` to be reassigned
x = 6;
```

**Restriction:** `mut` must appear at the binding site (`let mut x`); it
cannot be added later to make an already-immutable binding mutable.

## Embedded Rust Notes

**Full support.** `mut` is core grammar. It's used constantly in embedded
code for `&mut` access to peripheral registers and driver state — no
`std` dependency at all.
