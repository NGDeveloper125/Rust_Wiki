---
title: "Unit structs"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling"]
related_syntax: [struct]
see_also: ["Structs", "Zero-sized types & PhantomData"]
---

## Explanation

A unit struct has no fields at all:

```
struct Marker;
```

It carries no data — its only purpose is to *exist as a distinct type*,
usually so it can implement a trait or serve as a marker/tag. Because it
holds no data, a unit struct occupies zero bytes at runtime (see
[zero-sized types](zero-sized-types-phantomdata.md)) — using one costs
nothing beyond what the type system tracks at compile time, which is
exactly the point: it lets you encode a piece of information ("this
value is specifically an instance of `Marker`") purely in the type,
enforced by the compiler, with no runtime representation to go with it.

## Embedded Rust Notes

**Full support.** Zero-sized and allocator-free — embedded HAL crates use
unit structs constantly as typestate markers (e.g. a GPIO pin's mode
encoded as a zero-cost marker type).
