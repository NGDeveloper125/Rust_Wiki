---
title: "Default trait methods"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Object-Oriented-ish Patterns"]
related_syntax: [trait]
see_also: ["Traits", "Supertraits"]
---

## Explanation

A trait method can carry a default body, used by any implementer that
doesn't override it:

```
trait Greet {
    fn name(&self) -> String;
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}
```

Here every implementer of `Greet` must define `name`, but gets `greet`
for free unless it chooses to override it. This lets a trait provide
substantial shared behavior — not just a bare contract — while still
letting each implementer customize any specific piece of it, similar to
how an abstract base class in an OO language can implement some methods
concretely while leaving others abstract, but without requiring an actual
inheritance relationship between the types involved.

## Embedded Rust Notes

**Full support.** No allocator dependency — default methods work
identically in `#![no_std]`.
