---
title: "The borrow checker"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Unique to Rust", "Coming from Python / JavaScript", "Coming from Java / C#", "Coming from C / C++"]
related_syntax: ["&", mut]
see_also: ["Ownership", "Borrowing (shared references)", "Mutable borrowing", "Lifetimes"]
---

## Explanation

The borrow checker is the part of the compiler that statically verifies
every ownership and borrowing rule holds for every possible execution
path, before the program ever runs: no value is used after it's moved, no
reference outlives the data it points to, and no mutable reference
coexists with any other reference to the same data.

This is what most people mean when they talk about "fighting the borrow
checker" as a newcomer — it rejects programs that would be memory-unsafe
(dangling pointers, data races, use-after-free) at compile time, with no
runtime cost and no possibility of the bug reaching production, in
exchange for sometimes requiring the code to be restructured in ways that
feel unfamiliar coming from a language without this check.

The mental model that helps most: the borrow checker isn't an arbitrary
obstacle, it's checking a real property your program needs to hold
regardless of language — other languages either enforce a version of it
at runtime (garbage collection sidesteps use-after-free by never freeing
early; a mutex enforces exclusivity at runtime with a lock) or don't
enforce it at all (raw pointers in C, where violating it is undefined
behavior). Rust's distinguishing choice is doing the check entirely at
compile time, for zero runtime cost, which is only possible because the
rules are conservative — some genuinely safe programs are rejected simply
because the checker can't prove they're safe, which is why escape hatches
like [interior mutability](interior-mutability.md), reference counting,
and (in the rare, unavoidable case) `unsafe` exist.

## Embedded Rust Notes

**Full support.** The borrow checker runs at compile time on the host
toolchain regardless of what target you're compiling for — it applies
identically whether the output binary targets a desktop or a
Cortex-M microcontroller.
