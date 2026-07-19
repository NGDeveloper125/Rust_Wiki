---
title: "Marker traits (Send, Sync, Sized, Copy)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism"]
related_syntax: []
see_also: ["Send & Sync", "Copy vs Clone", "Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

A marker trait has no methods at all — implementing it adds no new
behavior to a type. Its entire purpose is to let the compiler (and other
trait bounds) know a type has some property, purely through the type
system:

- `Copy` marks a type as safe to duplicate with a simple bitwise copy
  (see [Copy vs Clone](../ownership-borrowing/copy-vs-clone.md)).
- `Sized` marks a type whose size is known at compile time — true for
  almost everything, which is why it's an implicit bound on generic
  parameters by default (opted out of with `?Sized`).
- `Send` marks a type as safe to move to another thread.
- `Sync` marks a type as safe to access from multiple threads at once
  through a shared reference (see [Send & Sync](../concurrency-async/send-and-sync.md)).

Most marker traits are auto-derived by the compiler for any type made
entirely of parts that already have the marker — a struct of `Send`
fields is automatically `Send` itself, with no `impl` needed. This is
what lets the compiler enforce thread-safety rules across an entire
program's type graph without every single type needing an explicit
annotation; the property propagates structurally, and only types doing
something genuinely unusual (raw pointers, certain FFI types) need to
opt out explicitly.

## Embedded Rust Notes

**Full support.** All defined in `core` — no `std` dependency. `Send`/`Sync`
are especially relevant in embedded code sharing state between a main
loop and an interrupt handler, or between tasks in an async executor like
`embassy`.
