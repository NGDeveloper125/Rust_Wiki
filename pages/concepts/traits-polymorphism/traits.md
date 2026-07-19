---
title: "Traits"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Object-Oriented-ish Patterns", "Decoupling", "Coming from Haskell / functional languages"]
related_syntax: [trait, impl]
see_also: ["Trait bounds", "Trait objects & dynamic dispatch (dyn Trait)", "Default trait methods"]
---

## Explanation

A trait defines a set of behavior — methods a type promises to provide —
that any type can then implement, independent of that type's own
definition. It's Rust's answer to "shared behavior across unrelated
types," filling the role interfaces play in Java/C#, protocols in Swift,
or type classes in Haskell.

```
trait Greet {
    fn greet(&self) -> String;
}
impl Greet for Cat {
    fn greet(&self) -> String { "meow".into() }
}
```

Because traits are implemented separately from a type's definition (via
`impl Trait for Type`, potentially in a completely different module or
even a different crate than the type itself), a type can implement any
number of unrelated traits without any of them needing to know about each
other — this is the mechanism behind
[decoupling](../../concepts-technique-topic-placeholder.md) code from
concrete types: a function can depend on "anything implementing this
trait" instead of a specific concrete type, without the trait author and
the type author ever needing to coordinate directly.

Traits are also how Rust achieves polymorphism without classical
inheritance — see
[trait objects & dynamic dispatch](trait-objects-dynamic-dispatch.md) for
the runtime-polymorphism side, and
[static dispatch & monomorphization](static-dispatch-monomorphization.md)
for the compile-time side.

## Embedded Rust Notes

**Full support.** Traits are core-language and allocator-free. The
`embedded-hal` crate is the clearest real-world proof of how central this
is to embedded Rust: an entire ecosystem of hardware-agnostic driver
crates is built purely on trait abstractions, letting one driver crate
work across dozens of unrelated vendors' microcontrollers.
