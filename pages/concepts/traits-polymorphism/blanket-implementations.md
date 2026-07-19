---
title: "Blanket implementations"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Writing Generic & Reusable Code", "Decoupling"]
related_syntax: []
see_also: ["Trait bounds", "Traits", "Generics as type classes"]
---

## Explanation

A blanket implementation implements a trait for every type that satisfies
some bound, all at once, rather than one type at a time:

```
impl<T: Display> ToString for T {
    // every Display type gets ToString for free
}
```

This is how, for example, every type implementing `Display` automatically
gets `.to_string()` in the standard library — the blanket impl covers the
entire (open-ended, unbounded) set of types satisfying `Display`, present
and future, without the standard library needing to know about any of
them individually.

Blanket impls are a powerful decoupling tool: a trait author can provide
functionality for "any type with property X" without ever enumerating
which types that includes, and library consumers get the functionality
automatically the moment their own type satisfies the bound — no explicit
opt-in `impl` required on their part. The tradeoff is that blanket impls
interact with [the orphan rule](the-orphan-rule-and-coherence.md) in
restrictive ways (only the crate defining the trait can write a blanket
impl for it), specifically to prevent two different crates from
providing conflicting blanket impls for the same trait.

## Basic usage example

```
trait Describe {
    fn describe(&self) -> String;
}

impl<T: std::fmt::Display> Describe for T { // <- one impl covers every Display type at once
    fn describe(&self) -> String {
        format!("value: {self}")
    }
}

println!("{}", 5.describe());
```

**Restriction:** only the crate that defines `Describe` may write this
blanket impl — the orphan rule forbids a downstream crate from
blanket-implementing someone else's trait for someone else's types.

## Embedded Rust Notes

**Full support.** No `std`/allocator dependency — the mechanism is purely
compile-time trait resolution.
