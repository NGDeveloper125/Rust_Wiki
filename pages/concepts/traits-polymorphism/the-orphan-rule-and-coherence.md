---
title: "The orphan rule & coherence"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Unique to Rust"]
related_syntax: []
see_also: ["The newtype pattern", "Blanket implementations", "Traits"]
---

## Explanation

The orphan rule says you may only implement a trait for a type if either
the trait or the type is defined in your own crate. You cannot, from a
third crate, implement a foreign trait (say, `std`'s `Display`) for a
foreign type (say, `Vec<T>`) — both are "orphans" to your crate, and the
rule forbids it.

This exists to guarantee **coherence**: for any given `(Trait, Type)`
pair, there is exactly one implementation in the entire program, ever,
with no possibility of two different crates each providing a conflicting
one. Without this guarantee, adding a dependency could silently change
which implementation of a trait gets used somewhere else in your
program, or the compiler could face a genuine ambiguity it has no
principled way to resolve — coherence is what lets trait resolution be a
single, unambiguous, whole-program-wide answer rather than something that
depends on which crates happen to be linked in.

The practical consequence is [the newtype pattern](../types-data-modeling/the-newtype-pattern.md):
if you need to implement a foreign trait for a foreign type, wrapping the
foreign type in your own newtype gives you a type that isn't an orphan,
which you're then free to implement anything on.

## Basic usage example

```
struct Meters(f64); // <- local type, so a foreign trait may be implemented on it

impl std::fmt::Display for Meters { // allowed: Meters is defined in this crate
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}m", self.0)
    }
}
```

**Restriction:** `impl std::fmt::Display for Vec<f64>` directly would be
rejected — both `Display` and `Vec` are foreign to this crate. The fix is
[the newtype pattern](../types-data-modeling/the-newtype-pattern.md): wrap
`Vec<f64>` in a local type like `Meters` above, then implement on that.

## Embedded Rust Notes

**Full support.** A compile-time-only rule enforced identically regardless
of target — no `std` dependency.
