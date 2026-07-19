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

## Basic usage example

```
struct Marker; // <- no fields: exists only to be a distinct type

trait Tag {}
impl Tag for Marker {} // <- typically exists so it can implement a trait

let _m = Marker; // zero bytes at runtime
```

## Best practices & deeper information

### Scenario: Implementing traits

A unit struct works well as a pure trait-only tag: it carries no data of
its own, exists only so the type system can distinguish "a door in the
locked state" from "a door in the unlocked state" at compile time.

```
struct Locked;   // <- marker types: no fields, exist only to be distinct types
struct Unlocked;

trait DoorState {}
impl DoorState for Locked {}
impl DoorState for Unlocked {} // <- each unit struct just needs to exist to satisfy the trait

fn describe<S: DoorState>(_state: S) -> &'static str {
    "handling a door in some DoorState"
}

describe(Unlocked); // <- constructing the marker value is free: zero bytes at runtime
```

**Why this way:** this is the typestate pattern from the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/patterns/behavioural/typestate.html)
book — encoding state as a marker type makes illegal states impossible to
construct in the first place, rather than merely checked at runtime, and
costs nothing beyond what the type system already tracks at compile time;
see [Zero-sized types & PhantomData](zero-sized-types-phantomdata.md) for
why that cost really is zero.

## Embedded Rust Notes

**Full support.** Zero-sized and allocator-free — embedded HAL crates use
unit structs constantly as typestate markers (e.g. a GPIO pin's mode
encoded as a zero-cost marker type).
