---
title: "Mutable borrowing"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing"]
related_syntax: ["&", mut]
see_also: ["Borrowing (shared references)", "The borrow checker", "Interior mutability (Cell & RefCell)"]
---

## Explanation

A mutable reference (`&mut T`) grants temporary, exclusive write access to
a value without transferring ownership. Unlike a shared reference, only
one `&mut T` to a given value can exist at a time, and while it exists, no
other reference — shared or mutable — to that same value may exist
alongside it.

This exclusivity is the mechanism that rules out data races at compile
time: two threads (or even two pieces of code on the same thread) can
never simultaneously read-while-writing or write-while-writing the same
data, because the compiler statically guarantees a `&mut T` is always
alone. This rule is often summarized as "aliasing XOR mutability" — a
value can have multiple readers *or* one writer, never both at once.

The restriction can feel strict when you genuinely need shared, mutable
access (a cache multiple parts of a program update, a graph with back
edges) — that's precisely the gap
[interior mutability](interior-mutability.md) (`Cell`/`RefCell`, and their
thread-safe counterparts `Mutex`/`RwLock`) exists to fill, by moving the
exclusivity check from compile time to run time in a controlled way.

## Embedded Rust Notes

**Full support.** No allocator or `std` dependency. The exclusivity
guarantee is especially valuable for embedded code touching shared
peripheral state from both a main loop and an interrupt handler — it's
part of what a sound embedded Rust design (e.g. RTIC's resource model)
leans on to rule out data races between them at compile time.
