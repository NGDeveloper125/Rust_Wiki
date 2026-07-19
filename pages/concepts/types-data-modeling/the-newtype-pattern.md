---
title: "The newtype pattern"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Designing Robust Data Models", "Unique to Rust"]
related_syntax: [struct]
see_also: ["Tuple structs", "Type aliases", "The orphan rule & coherence"]
---

## Explanation

The newtype pattern wraps an existing type in a single-field tuple struct
to give it a distinct identity:

```
struct Meters(f64);
struct Seconds(f64);
```

Even though both wrap `f64`, `Meters` and `Seconds` are different types
to the compiler — passing a `Seconds` value where `Meters` is expected is
a compile error, catching a whole category of unit-confusion bugs
(the kind that has, famously, destroyed real spacecraft) that a plain
type alias (which is fully interchangeable with what it aliases) would
not catch at all.

Newtypes serve a second, unrelated purpose too: Rust's
[orphan rule](../traits-polymorphism/the-orphan-rule-and-coherence.md)
forbids implementing a foreign trait on a foreign type directly (e.g. you
can't `impl Display for Vec<T>` from outside the crate that defines
either), but wrapping the foreign type in your own newtype gives you a
type you *do* own, on which you're free to implement whatever traits you
like. This combination — meaningful type safety plus a way around the
orphan rule — is common enough that it's considered one of Rust's
signature idioms rather than a niche trick.

## Embedded Rust Notes

**Full support.** No allocator dependency — this is one of the most-used
patterns in embedded HAL crates specifically, wrapping raw register
values and pin numbers in distinct types to prevent unit-confusion bugs
(millivolts vs. raw ADC counts, for example).
