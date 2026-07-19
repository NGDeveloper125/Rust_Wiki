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

## Best practices & deeper information

### Scenario: Designing a public API

Needing to implement a foreign trait (`Display`) for a foreign type
(`Vec<f64>`) is blocked by the orphan rule — the workaround is a local
newtype, which is a genuinely local type the rule permits implementing
anything on.

```
struct Readings(Vec<f64>); // <- local newtype: turns a foreign Vec<f64> into a type this crate owns

impl std::fmt::Display for Readings { // allowed: Readings is local, even though Display isn't
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

println!("{}", Readings(vec![1.0, 2.5, 3.75]));
```

**Why this way:** the newtype costs one wrapper type in exchange for full
freedom to implement anything on it — the
[Rust Design Patterns book](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
documents this as the standard route around the orphan rule, rather than
forking or vendoring the foreign crate.

### Scenario: Converting between types

Once data lives inside a newtype, `From`/`Into` make moving between the
newtype and the underlying foreign type ergonomic, instead of exposing
the wrapped field directly.

```
struct Readings(Vec<f64>);

impl From<Vec<f64>> for Readings { // <- lets `.into()` build a Readings from a plain Vec<f64>
    fn from(values: Vec<f64>) -> Self {
        Readings(values)
    }
}

let readings: Readings = vec![1.0, 2.5, 3.75].into(); // <- conversion, not direct field access
```

**Why this way:** the
[API Guidelines' C-CONV-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html)
recommend `From`/`Into` specifically so a newtype composes with any code
already written against the standard conversion traits, instead of
requiring callers to learn a bespoke constructor name.

## Embedded Rust Notes

**Full support.** A compile-time-only rule enforced identically regardless
of target — no `std` dependency.
