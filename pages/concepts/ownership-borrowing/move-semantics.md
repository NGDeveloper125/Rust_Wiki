---
title: "Move semantics"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Move Semantics", "Unique to Rust", "Coming from C / C++"]
related_syntax: [move, "="]
see_also: ["Ownership", "Copy vs Clone"]
---

## Explanation

Assigning a value to a new variable, passing it to a function, or
returning it from one transfers ownership rather than copying it by
default — this is a **move**. After a move, the original binding is no
longer valid; the compiler tracks this and rejects any later use of it as
a compile error, not a runtime bug.

```
let a = String::from("hi");
let b = a;      // ownership moves from a to b
// using `a` here is a compile error: value moved
```

This is a deliberate departure from two more familiar defaults: it's not
implicit reference/pointer semantics (as in Python, Java, JS, where
assignment shares the same object and mutation is visible through every
reference to it), and it's not implicit copying (as in C++, where
`Foo b = a;` invokes a copy constructor by default, unless you explicitly
write `std::move(a)`). Rust flips the C++ default: moving is the norm,
and copying only happens when a type explicitly opts in via `Copy` (see
[Copy vs Clone](copy-vs-clone.md)) or you call `.clone()` yourself.

The benefit is that "who owns this, and is it still valid here" is always
statically knowable and enforced by the compiler — there's no way to
accidentally hold onto and use a value that's already been logically
handed off elsewhere, a whole category of bug (use-after-move,
double-free) that move semantics eliminates by construction rather than
by convention or discipline.

## Basic usage example

```
fn consume(s: String) {
    println!("{s}");
} // s is dropped here, at the end of consume's scope

let a = String::from("hi");
consume(a); // <- ownership of `a` moves into the function call
// println!("{a}"); // would fail to compile: `a` was moved
```

## Embedded Rust Notes

**Full support.** Move semantics are core-language and allocator-free —
identical behavior in `#![no_std]`.
