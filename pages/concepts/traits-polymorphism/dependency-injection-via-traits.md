---
title: "Dependency injection via traits/generics"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Decoupling"]
related_syntax: []
see_also: ["Traits", "Trait bounds", "Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

Without a class hierarchy or a dependency-injection framework, Rust
achieves the same decoupling goal — code depending on an abstraction
rather than a concrete implementation — through trait bounds and trait
objects directly:

```
fn run(logger: &impl Logger) { logger.log("started"); }
// or, if the concrete type must vary at runtime:
fn run(logger: &dyn Logger) { logger.log("started"); }
```

`run` depends only on "something implementing `Logger`" — a test can pass
in a mock implementation, production code can pass in a real one, and
neither `run` nor `Logger` needs to know which concrete types will ever
implement it. This is the same inversion-of-control idea dependency
injection frameworks in other languages provide via containers and
runtime wiring, but here it's expressed directly in the function
signature and checked entirely at compile time (for `impl Trait`/generic
bounds) or resolved via a simple vtable (for `dyn Trait`) — no separate
framework, reflection, or runtime container needed.

## Embedded Rust Notes

**Full support.** No allocator dependency — this is precisely the pattern
`embedded-hal` is built around: application/driver code is generic over
a trait (e.g. a GPIO or SPI trait), decoupled from which vendor's
concrete peripheral implementation is plugged in at the top level.
