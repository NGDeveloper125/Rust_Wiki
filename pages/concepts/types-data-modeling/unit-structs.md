---
title: "Unit structs"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling"]
related_syntax: [struct]
see_also: ["Structs", "Zero-sized types & PhantomData"]
---

## Explanation

A unit struct has no fields at all — `struct Marker;` is a complete
definition on its own.

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

**Why this way:** this is the typestate pattern, covered in the
[Embedded Rust Book's Typestate Programming chapter](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html)
— encoding state as a marker type makes illegal states impossible to
construct in the first place, rather than merely checked at runtime, and
costs nothing beyond what the type system already tracks at compile time;
see [Zero-sized types & PhantomData](zero-sized-types-phantomdata.md) for
why that cost really is zero.

## Explanation (Embedded)

Unit structs are the standard building block for the typestate pattern
that runs throughout the `embedded-hal` ecosystem: a zero-field marker
type like `struct Configured;` or `struct Unconfigured;` exists purely so
the type system can tell "this peripheral has been set up" apart from
"this peripheral hasn't," with the marker itself contributing nothing to
a struct's runtime size. Paired with a generic parameter (see
[Zero-sized types & PhantomData](zero-sized-types-phantomdata.md) for how
a marker like this attaches to a generic type at zero cost), a unit
struct lets a driver expose methods that only exist for the
correctly-configured state — calling `read()` on an unconfigured
peripheral becomes a compile error rather than a runtime failure
discovered mid-flight. This matters more on embedded targets than almost
anywhere else: a runtime "peripheral not configured" check costs cycles
and flash on every call, and on hardware there's often no good recovery
path once the mistake is made — so moving the check to compile time, for
free, is a genuine win rather than a stylistic preference.

## Basic usage example (Embedded)

```
struct Configured;   // <- no fields: exists only to be a distinct, zero-sized type
struct Unconfigured;

struct Adc<State> {
    channel: u8,
    _state: core::marker::PhantomData<State>,
}

impl Adc<Unconfigured> {
    fn configure(self) -> Adc<Configured> { // <- consumes the unconfigured value, returns a configured one
        Adc { channel: self.channel, _state: core::marker::PhantomData }
    }
}

impl Adc<Configured> {
    fn read(&self) -> u16 { 0 /* real code would read the ADC register here */ }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Implementing traits

A GPIO pin's mode is a natural fit for unit-struct markers: `Input` and
`Output` each need to exist only long enough to implement a marker trait
the pin's generic type is bounded by.

```
struct Input;   // <- marker types: no fields, exist only to be distinct types
struct Output;

trait PinMode {}
impl PinMode for Input {}
impl PinMode for Output {} // <- each marker just needs to exist to satisfy the trait bound

fn configure<M: PinMode>(_mode: M) {
    // generic over which mode marker was supplied
}

configure(Output); // <- constructing the marker is free: zero bytes at runtime
```

**Why this way:** encoding a pin's mode as a marker type — rather than an
enum field checked at runtime — moves "is this pin in the right mode for
this operation" from a runtime branch (which costs cycles on every call)
to a compile-time fact the type system already guarantees, at zero
additional runtime representation.

### Scenario: Writing generic code

A driver generic over a marker parameter can offer methods that only
exist for one state, which is exactly how `embedded-hal`-style pin types
make misusing a peripheral a compile error instead of a runtime one.

```
struct Locked;
struct Unlocked;

struct FlashRegion<State> {
    base_addr: u32,
    _state: core::marker::PhantomData<State>,
}

impl FlashRegion<Locked> {
    fn unlock(self, key: u32) -> FlashRegion<Unlocked> { // <- consumes Locked, returns Unlocked
        let _ = key;
        FlashRegion { base_addr: self.base_addr, _state: core::marker::PhantomData }
    }
}

impl FlashRegion<Unlocked> {
    fn erase_page(&mut self, page: u32) { /* ... */ }
    // erase_page() simply doesn't exist on FlashRegion<Locked> -- calling it on
    // a locked region is a compile error, not a runtime hazard
}
```

**Why this way:** flash-erase operations on real hardware are genuinely
destructive and often need an explicit unlock sequence first — encoding
that sequence in the type via zero-sized markers (see
[Zero-sized types & PhantomData](zero-sized-types-phantomdata.md) for why
the marker parameter is free) means the compiler, not a runtime guard,
rejects the erase-before-unlock mistake. See also
[The typestate pattern](../design-patterns-idioms/the-typestate-pattern.md)
for the general pattern this technique belongs to.
