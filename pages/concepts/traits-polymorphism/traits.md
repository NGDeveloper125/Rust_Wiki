---
title: "Traits"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Object-Oriented-ish Patterns", "Decoupling", "Coming from Haskell / functional languages"]
related_syntax: [trait, impl]
see_also: ["Trait bounds", "Trait objects & dynamic dispatch (dyn Trait)", "Default trait methods"]
---

## Explanation

A trait defines a set of behavior — methods a type promises to provide —
that any type can then implement, independent of that type's own
definition. It's Rust's answer to "shared behavior across unrelated
types," filling the role interfaces play in Java/C#, protocols in Swift,
or type classes in Haskell. For instance, a `Greet` trait declaring a
`greet` method can be implemented for a `Cat` struct with `impl Greet for
Cat`.

Because traits are implemented separately from a type's definition (via
`impl Trait for Type`, potentially in a completely different module or
even a different crate than the type itself), a type can implement any
number of unrelated traits without any of them needing to know about each
other — this is the mechanism behind
[decoupling](dependency-injection-via-traits.md) code from
concrete types: a function can depend on "anything implementing this
trait" instead of a specific concrete type, without the trait author and
the type author ever needing to coordinate directly.

Traits are also how Rust achieves polymorphism without classical
inheritance — see
[trait objects & dynamic dispatch](trait-objects-dynamic-dispatch.md) for
the runtime-polymorphism side, and
[static dispatch & monomorphization](static-dispatch-monomorphization.md)
for the compile-time side.

## Basic usage example

```
trait Greet {
    fn greet(&self) -> String;
}

struct Cat;
impl Greet for Cat { // <- Cat implements the Greet trait
    fn greet(&self) -> String { "meow".into() }
}

println!("{}", Cat.greet());
```

## Best practices & deeper information

### Scenario: Implementing traits

A domain type gains broad interoperability by implementing a standard
trait like `Display`, rather than a bespoke method — any code already
written against `Display` accepts it for free.

```
struct Temperature { celsius: f64 }

impl std::fmt::Display for Temperature { // <- implementing a trait, not a one-off to_string() method
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.1}°C", self.celsius)
    }
}

fn log(value: &impl std::fmt::Display) { // <- accepts anything implementing Display
    println!("reading: {value}");
}

log(&Temperature { celsius: 21.5 });
```

**Why this way:** eagerly implementing common standard traits lets a type
slot directly into generic code, formatting macros, and collections that
already know how to work with them, instead of every caller needing a
type-specific method — the
[API Guidelines' C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html)
codifies this.

### Scenario: Designing a public API

Shipping a trait as a library's extension point, instead of a concrete
struct, is a deliberate design choice — callers implement the trait for
their own types instead of being locked into the crate's.

```
pub trait Storage { // <- the crate's extension point is a trait, not a fixed struct
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: String);
}

pub struct Cache<S: Storage> {
    backend: S,
}

impl<S: Storage> Cache<S> {
    pub fn refresh(&mut self, key: &str, value: String) {
        self.backend.set(key, value);
    }
}
```

**Why this way:** keeping the trait narrow (two methods, nothing about how
storage is implemented) leaves room to add backends later without
breaking existing implementers — see the
[API Guidelines on future-proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html)
for the broader case around designing traits as stable extension points.

### Scenario: Testing

Defining a trait around a dependency — even one with a single real
implementation — opens the door to a test-only stand-in, without any
conditional compilation inside the production code path.

```
trait Clock {
    fn now(&self) -> u64;
}

struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> u64 { /* reads the real system time */ 1_700_000_000 }
}

struct FixedClock(u64); // <- test-only implementation of the same trait
impl Clock for FixedClock {
    fn now(&self) -> u64 { self.0 }
}

fn is_expired(clock: &impl Clock, expiry: u64) -> bool {
    clock.now() > expiry
}

assert!(is_expired(&FixedClock(2_000_000_000), 1_000_000_000)); // no real clock involved
```

**Why this way:** depending on the trait rather than `SystemClock` directly
means the test controls time deterministically instead of racing the real
clock — a standard use of trait seams for testability, as the
[Rust Book's mock-object example](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#a-use-case-for-interior-mutability-mock-objects)
illustrates with the same trait-double approach.

## Explanation (Embedded)

Traits are the mechanism that makes `embedded-hal` possible, and
`embedded-hal` is the single clearest proof of how central traits are to
embedded Rust. `embedded-hal` (and its async sibling, `embedded-hal-async`)
ships almost no implementation code at all — it's a collection of trait
definitions (`OutputPin`, `InputPin`, `SpiBus`, `I2c`, `DelayNs`, …), full
stop. See [`trait`](../../syntax/keywords/trait.md) for the mechanics of
how those declarations are written; this page is about the design payoff.

Because a trait separates "the contract" from "who implements it," a chip
vendor's HAL crate (typically layered over a peripheral-access crate
generated from that chip's SVD file) can implement `embedded-hal`'s traits
for its own concrete register types, while a sensor or display driver
crate is written entirely against the traits — never against any vendor's
concrete pin, bus, or timer type. Neither side has to know about the
other in advance: the trait is the sole point of coordination. The result
is an ecosystem where one driver crate (say, a BME280 humidity sensor
driver) compiles and runs unmodified on an STM32 board, an RP2040 board,
and a nRF52 board, purely because all three vendors' HALs implement the
same `embedded-hal` traits. Without traits, this would require either
per-vendor forks of every driver, or a runtime abstraction layer with the
dispatch and code-size costs that come with it — traits get the same
decoupling for free, resolved entirely at compile time.

## Basic usage example (Embedded)

```
trait OutputPin {
    type Error;
    fn set_high(&mut self) -> Result<(), Self::Error>;
}

struct GpioPin5; // a concrete pin type, as a vendor's HAL crate would define it

impl OutputPin for GpioPin5 { // <- the vendor HAL implements the shared trait for its own pin type
    type Error = core::convert::Infallible;
    fn set_high(&mut self) -> Result<(), Self::Error> {
        // write to the peripheral's GPIO register here
        Ok(())
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Implementing traits

A vendor HAL crate makes its concrete I2C peripheral usable by every
`embedded-hal`-based driver crate simply by implementing the shared `I2c`
trait for its own bus type — no coordination with any driver author
required.

```
trait I2c {
    type Error;
    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error>;
}

struct Bus1; // the vendor's concrete I2C peripheral handle

impl I2c for Bus1 { // <- one impl unlocks every driver crate written against I2c
    type Error = core::convert::Infallible;
    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        let _ = (addr, bytes); // send over the real peripheral here
        Ok(())
    }
}
```

**Why this way:** the HAL author and every driver author never need to
meet — the trait is the entire coordination surface, which is exactly why
`embedded-hal` chose "implement these traits" as its integration model
instead of a shared concrete bus type.

### Scenario: Designing a public API

A sensor driver crate's public constructor should ask for "anything
implementing the `embedded-hal` trait it needs," not a specific vendor's
concrete peripheral type — otherwise the driver silently becomes
single-vendor.

```
pub struct TemperatureSensor<I2C> { // <- generic over the trait, not any one vendor's I2C type
    bus: I2C,
}

impl<I2C: I2c> TemperatureSensor<I2C> {
    pub fn new(bus: I2C) -> Self {
        TemperatureSensor { bus }
    }
}
```

**Why this way:** a driver crate that names a concrete vendor type in its
public API can only ever be used with that vendor's chips; bounding by
the trait instead is what lets the same driver crate serve every board
whose HAL implements `I2c`.

### Scenario: Testing

A sensor driver's trait boundary doubles as a test seam: implementing the
same trait with an in-memory stand-in lets the driver's parsing/decoding
logic run in a regular host-side test, with no real bus and no target
hardware attached.

```
struct MockBus { canned_reply: Vec<u8> }

impl I2c for MockBus {
    type Error = core::convert::Infallible;
    fn write(&mut self, _addr: u8, _bytes: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}

// TemperatureSensor::new(MockBus { canned_reply: vec![0x1a, 0x02] }) exercises
// the driver's own logic without ever touching real hardware
```

**Why this way:** this is precisely the role the `embedded-hal-mock` crate
plays in the ecosystem — driver crates are tested against a mock
implementation of the same traits real hardware satisfies, so CI can run
a board's driver logic without the board.
