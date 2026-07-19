---
title: "Supertraits"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Writing Generic & Reusable Code"]
related_syntax: [trait, ":"]
see_also: ["Traits", "Trait bounds"]
---

## Explanation

A trait can require that any implementer also implement another trait —
its supertrait:

```
trait Named {
    fn name(&self) -> String;
}
trait Greet: Named {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}
```

Here `Greet: Named` means "you can only implement `Greet` if you've also
implemented `Named`" — which is what lets `Greet`'s default method call
`self.name()` at all, since that method is guaranteed to exist on any
type this trait is implemented for. This is Rust's closest equivalent to
interface inheritance, but it composes rather than creates a hierarchy:
a trait can have several supertraits, and unrelated traits can share the
same supertrait without any of them being related to each other beyond
that one shared requirement.

## Embedded Rust Notes

**Full support.** No `std`/allocator dependency.
