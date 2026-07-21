---
title: "Generics"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Writing Generic & Reusable Code", "Polymorphism", "Generic Programming"]
related_syntax: ["<", ">"]
see_also: ["Trait bounds", "Static dispatch & monomorphization", "Const generics"]
---

## Explanation

Generics let a type or function be written once and used with many
different concrete types, without duplicating the code for each one — a
function like `fn largest<T: PartialOrd>(items: &[T]) -> &T` works for any
type `T` that satisfies the bound, instead of needing a separate copy per
concrete type.

Here `T` stands for "some type, to be determined at each call site,"
constrained by a [trait bound](../traits-polymorphism/trait-bounds.md)
(`PartialOrd`, in this example) so the function body can rely on the
operations it actually needs. Generics are resolved entirely at compile
time — see
[static dispatch & monomorphization](../traits-polymorphism/static-dispatch-monomorphization.md)
— which means generic code has no inherent runtime overhead compared to
writing the same function by hand for each concrete type; the compiler
does that duplication for you, automatically, and specializes each copy
for its specific type.

This is the main way Rust achieves reusable, type-safe abstractions
without needing to fall back to dynamic typing or runtime type checks —
the compiler verifies, once, at the definition site plus every call site,
that every operation the generic code performs is valid for whatever
concrete type ends up substituted in.

## Basic usage example

```
fn largest<T: PartialOrd>(items: &[T]) -> &T {
//        ^^^^^^^^^^^^^^ T is a generic type, constrained to types that support `>`
    let mut best = &items[0];
    for item in items {
        if item > best { best = item; }
    }
    best
}

largest(&[3, 7, 2]);       // T = i32 here
largest(&[1.5, 0.2]);      // T = f64 here, same function definition
```

## Best practices & deeper information

### Scenario: Writing generic code

The trait bounds on a generic parameter should ask for exactly the
operations the function body uses — no more — so the function stays
usable with the widest reasonable range of types.

```
fn largest<T: PartialOrd + Copy>(items: &[T]) -> T {
//        ^^^^^^^^^^^^^^^^^^^^^ bounded to only what the body needs: ordering and cheap copies
    let mut best = items[0];
    for &item in &items[1..] {
        if item > best {
            best = item;
        }
    }
    best
}

let highest_temp = largest(&[21.5, 19.0, 24.3]);  // T = f64
let highest_score = largest(&[88, 95, 72]);        // T = i32, same function body
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-function-definitions)
covers bounding a type parameter to only what's used inside the body —
over-constraining with unused bounds (e.g. requiring `Clone` when
`Copy` already covers it) narrows which callers can use the function for
no benefit.

### Scenario: Working with collections

A small generic wrapper type, written once, can back many different
key/value pairings without duplicating the struct or its methods per
concrete type.

```
struct Cache<K, V> { // <- generic over both the key and value types, defined once
    entries: Vec<(K, V)>,
}

impl<K: PartialEq, V> Cache<K, V> {
    fn new() -> Self {
        Cache { entries: Vec::new() }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

let mut sensors: Cache<&str, f64> = Cache::new();
sensors.entries.push(("temp-1", 21.5));
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-struct-definitions)
covers generic struct definitions for exactly this case — `Cache<K, V>`
works for `Cache<&str, f64>`, `Cache<u32, String>`, or any other pairing
with no duplicated code, and monomorphization means each instantiation
runs exactly as fast as a hand-written version would.

## Explanation (Embedded)

Generic, trait-bounded code is arguably the single most load-bearing
pattern in the embedded Rust ecosystem: it's how a driver crate — an
accelerometer, a display, a flash chip — gets written *once* and works
across every microcontroller vendor's HAL, instead of one copy per chip
family. The `embedded-hal` crate defines a set of small, vendor-neutral
traits (`i2c::I2c`, `spi::SpiBus`, `digital::OutputPin`, …); a driver
writes its logic generic over `BUS: embedded_hal::i2c::I2c` rather than
over any concrete vendor's I2C peripheral type, and each vendor's HAL
crate (`stm32f4xx-hal`, `nrf52840-hal`, `rp2040-hal`, …) provides its own
concrete type implementing that same trait. The driver never mentions a
specific chip; the HAL crate never mentions the driver. Generics are what
let those two crates, written by different people for different hardware,
meet in the middle.

Because generics resolve entirely at compile time, this hardware-agnostic
abstraction costs nothing at runtime the way a `dyn Trait`/vtable
approach would: no indirect call, no vtable lookup, and the compiler can
inline across the trait boundary the same way it would for a
non-generic function, which matters for tightly-timed code (bit-banged
protocols, cycle-counted delays) where an indirect call's timing isn't
just slower but *less predictable*. The cost moves elsewhere instead:
monomorphization means the compiler emits one full specialized copy of
the generic driver code per concrete type it's instantiated with, so a
firmware image linking against several different concrete bus types
through the same generic driver pays in flash size for each copy — a
real and sometimes decisive constraint on a target with, say, 64 KB of
flash total. Choosing between a generic, monomorphized driver and a
`dyn Trait`-based one is frequently a code-size-vs-call-overhead
tradeoff made explicitly for that reason, not a default either way.

## Basic usage example (Embedded)

```
use embedded_hal::digital::OutputPin;

fn blink<PIN: OutputPin>(pin: &mut PIN) { // <- generic over any HAL's concrete GPIO output pin type
    pin.set_high().ok();
    pin.set_low().ok();
}

// Called identically whether `pin` is an stm32f4xx-hal, nrf52840-hal, or rp2040-hal pin type --
// `blink` never names a specific vendor, and monomorphizes to one specialized copy per type used.
```

## Best practices & deeper information (Embedded)

### Scenario: Writing generic code

A sensor driver's core logic — issuing the right register reads and
writes — has nothing to do with which vendor's I2C peripheral it runs on;
writing it generic over `embedded_hal::i2c::I2c` lets the same driver
crate compile against any HAL that implements the trait.

```
use embedded_hal::i2c::I2c;

struct TempSensor<BUS> { // <- generic over the bus type, not any one vendor's concrete I2C peripheral
    bus: BUS,
    address: u8,
}

impl<BUS: I2c> TempSensor<BUS> {
    fn read_celsius(&mut self) -> Result<f32, BUS::Error> {
        let mut raw = [0u8; 2];
        self.bus.write_read(self.address, &[0x00], &mut raw)?;
        Ok(i16::from_be_bytes(raw) as f32 / 256.0)
    }
}
```

**Why this way:** bounding `BUS` to exactly `embedded_hal::i2c::I2c` — no
more — means `TempSensor` compiles against any board's HAL crate that
implements that trait, without the driver crate depending on any single
vendor's hardware-access library; this is the standard shape of
essentially every driver crate published for `embedded-hal`.

### Scenario: Designing a public API

A driver crate's public API should be generic over the trait it actually
needs, not over a specific board's HAL type, and should weigh
monomorphized flash cost against `dyn Trait`'s runtime indirection
deliberately rather than defaulting to generics out of habit.

```
use embedded_hal::spi::SpiBus;

// PREFER (most drivers): generic + monomorphized -- zero call overhead, cost paid in flash per instantiation
pub struct FlashChip<BUS: SpiBus> {
    bus: BUS,
}

// AVOID unless flash budget forces it: dyn Trait -- one shared code path, but every call is now indirect
pub struct FlashChipDyn<'a> {
    bus: &'a mut dyn SpiBus<Error = core::convert::Infallible>,
}
```

**Why this way:** most embedded driver crates default to the generic
form because timing-sensitive bus transactions benefit from inlining and
predictable, non-indirect calls; a `dyn Trait` form is the right
trade only when a project links many different concrete bus
implementations through the same driver and flash size, not call
overhead, is the binding constraint — a decision worth making
explicitly rather than by default.
