---
title: "Trait bounds"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Writing Generic & Reusable Code", "Decoupling", "Generic Programming"]
related_syntax: [":", "+", where]
see_also: ["Traits", "Generics", "Supertraits"]
---

## Explanation

A trait bound constrains a generic type parameter to only the types that
implement a given trait, giving generic code exactly the guarantees it
needs to actually do something useful with a value of an otherwise
unknown type:

```
fn largest<T: PartialOrd>(items: &[T]) -> &T { ... }
```

Without the bound, the compiler would have no basis for allowing `>` to
be used on values of type `T` inside the function body — `T` could be
anything. The bound `T: PartialOrd` says "whatever `T` ends up being, it
must support ordering comparisons," which the compiler then checks holds
at every call site.

This is the mechanism that lets Rust decouple code from concrete types
in a fully type-checked way: a function can depend on "any type that can
be compared" or "any type that can be displayed," rather than a specific
concrete type, and the compiler verifies both that the generic code only
uses operations the bound actually grants, and that every real type
passed in at a call site genuinely satisfies the bound. Multiple bounds
combine with `+` (`T: Clone + Debug`), and a `where` clause is available
for bounds that get too long to read comfortably inline.

## Basic usage example

```
fn largest<T: PartialOrd>(items: &[T]) -> &T { // <- bound: only types supporting `>` allowed
    let mut m = &items[0];
    for item in items {
        if item > m { m = item; }
    }
    m
}

largest(&[3, 7, 2]);
```

## Embedded Rust Notes

**Full support.** A purely compile-time mechanism — no `std`/allocator
dependency, and central to how `embedded-hal`-based drivers stay generic
over any concrete peripheral implementation.
