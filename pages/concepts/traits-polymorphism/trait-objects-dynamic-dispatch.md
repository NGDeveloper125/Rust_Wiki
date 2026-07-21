---
title: "Trait objects & dynamic dispatch (dyn Trait)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Object-Oriented-ish Patterns", "Decoupling", "Polymorphism", "Type Erasure", "Coming from Java / C#"]
related_syntax: [dyn]
see_also: ["Static dispatch & monomorphization", "Traits", "On-stack dynamic dispatch"]
---

## Explanation

A trait object (`dyn Trait`) erases a value's concrete type, keeping only
that it implements a given trait — and resolves which implementation's
methods to call at runtime rather than compile time. Since its size isn't
known at compile time, a trait object almost always appears behind a
pointer: `&dyn Trait`, `Box<dyn Trait>`, `Rc<dyn Trait>`. For instance, a
`Vec<Box<dyn Shape>>` can hold `Circle`s and `Square`s side by side, and
calling `.area()` on each element resolves to the right implementation at
runtime via a vtable.

This is what lets a single collection hold values of genuinely different
concrete types, as long as they all implement the same trait — something
[generics](../types-data-modeling/generics.md) alone can't do, since a
generic function/type is monomorphized into one specialized version *per*
concrete type, not a single version that handles several types
interchangeably at once.

The mechanism is a vtable: a small table of function pointers built for
each concrete type's implementation, attached alongside the data pointer
whenever a trait object is created. Calling a method through `dyn Trait`
means an extra indirection through that table — a small, real runtime
cost compared to [static dispatch](static-dispatch-monomorphization.md),
which is the tradeoff being made in exchange for the ability to mix
different concrete types behind one interface. This is Rust's nearest
analogue to interface-typed references in Java/C#, though notably
without null being a possible value.

## Basic usage example

```
trait Shape {
    fn area(&self) -> f64;
}
struct Circle;
impl Shape for Circle { fn area(&self) -> f64 { 3.14 } }
struct Square;
impl Shape for Square { fn area(&self) -> f64 { 4.0 } }

let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle), Box::new(Square)];
// <- `dyn Shape` erases the concrete type; each element can be a different one
for s in &shapes {
    println!("{}", s.area()); // resolved at runtime via a vtable
}
```

## Best practices & deeper information

### Scenario: Runtime polymorphism

An audio effect chain needs to apply an arbitrary, user-configured mix of
effects in sequence — the exact set and order aren't known until runtime,
which rules out a fixed enum or a single generic function.

```
trait Effect {
    fn apply(&self, sample: f32) -> f32;
}

struct Gain(f32);
impl Effect for Gain {
    fn apply(&self, sample: f32) -> f32 { sample * self.0 }
}

struct Clip(f32);
impl Effect for Clip {
    fn apply(&self, sample: f32) -> f32 { sample.clamp(-self.0, self.0) }
}

fn process(sample: f32, chain: &[Box<dyn Effect>]) -> f32 { // <- one signature, any mix of effect types
    chain.iter().fold(sample, |s, effect| effect.apply(s))
}

let chain: Vec<Box<dyn Effect>> = vec![Box::new(Gain(2.0)), Box::new(Clip(1.0))];
process(0.6, &chain);
```

**Why this way:** the chain's length and composition are decided at
runtime (config, user input), so no single generic instantiation could
cover it — this is exactly the heterogeneous-collection case the
[Rust Book](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
uses trait objects for.

### Scenario: Designing a public API

A plugin-style registry is a natural `Vec<Box<dyn Trait>>` API: the crate
owns the collection and dispatch logic, while callers register any type
implementing the trait.

```
trait Command {
    fn name(&self) -> &str;
    fn run(&self, input: &str);
}

pub struct Registry {
    commands: Vec<Box<dyn Command>>, // <- registry stores any Command impl behind one type
}

impl Registry {
    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    pub fn dispatch(&self, name: &str, input: &str) {
        if let Some(cmd) = self.commands.iter().find(|c| c.name() == name) {
            cmd.run(input);
        }
    }
}
```

**Why this way:** designing `Command` to stay object-safe (no generic
methods, no returning `Self`) is what makes this API possible at all —
the [API Guidelines' C-OBJECT](https://rust-lang.github.io/api-guidelines/flexibility.html)
calls out object safety as a deliberate design goal for traits meant to be
used this way, and the shape matches the
[command pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html)
in the Rust Design Patterns book.

## Explanation (Embedded)

For the mechanics of `&dyn Trait` needing no heap, `Box<dyn Trait>`
needing `alloc`, and the flash-vs-dispatch-predictability tradeoff against
generics, see [`dyn`](../../syntax/keywords/dyn.md) — that page covers
the mechanism in full and this page doesn't re-derive it. What's worth
covering here is the design-level question: given that generics and
monomorphization are the default choice in embedded Rust (predictable
inlined calls, no vtable indirection), when does embedded code reach for
dynamic dispatch at all?

Two situations come up genuinely often. The first is a **heterogeneous
collection of different peripheral types that must live together in one
array or list** — a board-support struct holding several distinct sensor
types that all need to be read in a loop, where a generic function can't
help because a single monomorphized instantiation only ever handles one
concrete type, not a mix. The second is **plugin-style, configuration-driven
registration** — a board's revision or a runtime configuration decides
*which* optional peripherals are actually present, so the code iterating
over them can't be generic over a fixed, compile-time-known set of types.
In both cases, `&dyn Trait` (or, for owned storage, `Box<dyn Trait>` once
`alloc` is configured) gives one uniform type to store and iterate,
without requiring every element to be the same concrete type. Outside
those two shapes, embedded code defaults to generics — the common case
by a wide margin, since most driver code is written against one concrete
type parameter per call site, decided once at compile time.

## Basic usage example (Embedded)

```
trait Sensor {
    fn read_raw(&self) -> u16;
}

struct Thermistor;
impl Sensor for Thermistor { fn read_raw(&self) -> u16 { 512 } }

struct Photodiode;
impl Sensor for Photodiode { fn read_raw(&self) -> u16 { 128 } }

let thermistor = Thermistor;
let photodiode = Photodiode;
let sensors: [&dyn Sensor; 2] = [&thermistor, &photodiode]; // <- heterogeneous, heap-free: fixed-size array of trait objects
for s in sensors {
    let _raw = s.read_raw();
}
```

## Best practices & deeper information (Embedded)

### Scenario: Runtime polymorphism

A board's set of installed sensors varies by revision — one revision
carries a thermistor and a photodiode, another adds a humidity sensor —
so the firmware that polls "whatever sensors this board has" needs one
type it can iterate over, without a fixed enum or a generic function per
board revision.

```
trait Sensor {
    fn read_raw(&self) -> u16;
}

fn poll_all(sensors: &[&dyn Sensor]) -> u32 { // <- one signature, any mix of sensor types, no heap
    sensors.iter().map(|s| s.read_raw() as u32).sum()
}
```

**Why this way:** the exact sensor mix is a board-configuration fact, not
something a single generic instantiation can express — `&dyn Sensor`
gives one type for the collection while costing nothing but the vtable
indirection on each read, with no allocator involved at all.

### Scenario: Designing a public API

A board-support crate that lets applications register an arbitrary
number of optional peripheral drivers at startup needs a fixed-capacity,
allocator-free registry — a plain array of trait objects, sized to the
board's maximum, rather than a heap-growing `Vec`.

```
trait Peripheral {
    fn init(&mut self);
}

pub struct Registry<'a> {
    peripherals: [Option<&'a mut dyn Peripheral>; 4], // <- fixed-capacity, no allocator: registry sized at compile time
}

impl<'a> Registry<'a> {
    pub fn init_all(&mut self) {
        for p in self.peripherals.iter_mut().flatten() {
            p.init();
        }
    }
}
```

**Why this way:** a bare-metal target commonly has no `#[global_allocator]`
configured at all, so a registry API built around `Vec<Box<dyn Trait>>`
would be unusable there; sizing the registry to a fixed maximum at
compile time keeps the API allocator-free while still accepting any
concrete `Peripheral` implementation.

### Scenario: Boxing and heap allocation

On a target that *does* have `alloc` configured (an ESP32 running
`esp-idf`, for instance), a one-time, build-at-startup plugin list is a
reasonable place for `Box<dyn Trait>` — the allocation happens once
during init, not on a hot path, so the flexibility is worth the one-time
heap cost.

```
// AVOID: Box<dyn Trait> on a bare-metal target with no configured allocator — this won't compile
// let drivers: Vec<Box<dyn Peripheral>> = vec![Box::new(imu), Box::new(display)];

// PREFER: on a target with `alloc` configured, a startup-only Box<dyn Trait> list is fine
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

fn build_drivers(imu: impl Peripheral + 'static, display: impl Peripheral + 'static) -> Vec<Box<dyn Peripheral>> {
    let mut drivers: Vec<Box<dyn Peripheral>> = Vec::new();
    drivers.push(Box::new(imu));     // <- heap allocation, but only once, at startup
    drivers.push(Box::new(display));
    drivers
}
```

**Why this way:** whether `Box<dyn Trait>` is available at all is a
per-target fact, not a matter of taste — on targets without `alloc`,
`&dyn Trait` over a fixed-size array is the only option; where `alloc` is
configured, restricting `Box<dyn Trait>` use to one-time startup code
avoids paying an allocation cost on any latency-sensitive path.
