---
title: "Supertraits"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Writing Generic & Reusable Code"]
related_syntax: [trait, ":"]
see_also: ["Traits", "Trait bounds"]
---

## Explanation

A trait can require that any implementer also implement another trait —
its supertrait. For example, a `trait Greet: Named` declaration requires
any `Greet` implementer to also implement `Named`.

Here `Greet: Named` means "you can only implement `Greet` if you've also
implemented `Named`" — which is what lets `Greet`'s default method call
`self.name()` at all, since that method is guaranteed to exist on any
type this trait is implemented for. This is Rust's closest equivalent to
interface inheritance, but it composes rather than creates a hierarchy:
a trait can have several supertraits, and unrelated traits can share the
same supertrait without any of them being related to each other beyond
that one shared requirement.

## Basic usage example

```
trait Named {
    fn name(&self) -> String;
}

trait Greet: Named { // <- Greet requires Named too
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Cat;
impl Named for Cat { fn name(&self) -> String { "Cat".into() } }
impl Greet for Cat {} // only allowed because Cat also implements Named

println!("{}", Cat.greet());
```

## Best practices & deeper information

### Scenario: Implementing traits

The standard library itself leans on supertraits: `Eq` requires
`PartialEq`, so implementing `Eq` for a type is only possible once
`PartialEq` is implemented too.

```
#[derive(PartialEq)] // <- required first: Eq's supertrait
struct SensorId(u32);

impl Eq for SensorId {} // only allowed because SensorId already implements PartialEq

fn has_target<T: Eq>(ids: &[T], target: &T) -> bool { // <- Eq usable here because SensorId satisfies it
    ids.contains(target)
}

has_target(&[SensorId(1), SensorId(2)], &SensorId(2));
```

**Why this way:** `Eq` adds no methods of its own — it only asserts that
`PartialEq`'s equality is total (reflexive for every value) — which is
exactly the kind of "implementer promises an extra property" role a
supertrait exists to express; see
[`std::cmp::Eq`](https://doc.rust-lang.org/std/cmp/trait.Eq.html).

### Scenario: Writing generic code

A generic function bounded by a trait that has a supertrait can call the
supertrait's methods too, without adding a second bound — the supertrait
relationship already guarantees it's implemented.

```
trait Named {
    fn name(&self) -> String;
}
trait Reportable: Named { // <- Reportable's supertrait guarantees name() exists
    fn severity(&self) -> u8;
}

fn report<T: Reportable>(item: &T) -> String {
    format!("[{}] severity {}", item.name(), item.severity()) // <- name() usable with only a Reportable bound
}

struct DiskAlert;
impl Named for DiskAlert { fn name(&self) -> String { "disk".into() } }
impl Reportable for DiskAlert { fn severity(&self) -> u8 { 3 } }

report(&DiskAlert);
```

**Why this way:** writing `fn report<T: Reportable + Named>` would be
redundant — a supertrait bound is elaborated, so `T: Reportable` already
implies `T: Named` wherever it's required, as the
[Rust Reference's section on supertraits](https://doc.rust-lang.org/reference/items/traits.html#supertraits)
describes.

## Explanation (Embedded)

Supertraits show up throughout `embedded-hal`'s own trait hierarchy, not
just in illustrative examples: most of its device traits (`OutputPin`,
`InputPin`, `SpiBus`, `I2c`, …) require an `ErrorType` supertrait that
declares the associated `Error` type they use, so implementing, say,
`OutputPin` for a concrete pin type is only possible once `ErrorType` is
implemented for it too — the exact "implementer promises an extra piece
first" role a supertrait exists to express.

The same pattern appears one level up, in driver crates built *on*
`embedded-hal`. A crate providing a higher-level IMU driver trait might
declare `trait ImuDriver: SpiBus`, requiring any type implementing
`ImuDriver` to also implement `embedded-hal`'s `SpiBus` — guaranteeing the
driver trait's own default methods can call `self.transfer(...)` without
adding a second bound everywhere the trait is used. This is a common
shape for embedded driver crates that want to layer higher-level
behavior (parsing sensor registers, managing a device's power state) on
top of a raw bus capability, without re-declaring the bus methods
themselves.

## Basic usage example (Embedded)

```
trait SpiBus {
    type Error;
    fn transfer(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
}

trait ImuDriver: SpiBus { // <- ImuDriver requires SpiBus too
    fn read_accel_raw(&mut self) -> Result<[u8; 6], Self::Error> {
        let mut buf = [0u8; 6];
        self.transfer(&mut buf)?;
        Ok(buf)
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Implementing traits

`embedded-hal` itself leans on supertraits: its device traits require an
`ErrorType` supertrait declaring the associated `Error` type, so a
vendor's HAL crate must implement `ErrorType` for a pin type before
`OutputPin` can be implemented for it.

```
trait ErrorType { // <- embedded-hal's supertrait: declares the associated error type
    type Error;
}

trait OutputPin: ErrorType { // <- OutputPin requires ErrorType too
    fn set_high(&mut self) -> Result<(), Self::Error>;
}

struct GpioPin5;
impl ErrorType for GpioPin5 { type Error = core::convert::Infallible; } // required first
impl OutputPin for GpioPin5 { // only allowed because GpioPin5 already implements ErrorType
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
}
```

**Why this way:** splitting the associated error type into its own
`ErrorType` supertrait, rather than declaring it directly on `OutputPin`,
is what lets multiple `embedded-hal` traits (`OutputPin`, `InputPin`, …)
share the same associated `Error` type on one pin, instead of each trait
declaring a conflicting one of its own.

### Scenario: Writing generic code

A driver function generic over an `ImuDriver` implementation can call the
`SpiBus` methods `ImuDriver` requires without adding a second bound — the
supertrait relationship already guarantees `SpiBus` is implemented.

```
fn read_and_log<T: ImuDriver>(imu: &mut T) -> Result<(), T::Error> {
    let raw = imu.read_accel_raw()?;           // ImuDriver's own method
    let mut passthrough = [0u8; 1];
    imu.transfer(&mut passthrough)?;           // <- SpiBus's method, usable with only the ImuDriver bound
    let _ = raw;
    Ok(())
}
```

**Why this way:** writing `fn read_and_log<T: ImuDriver + SpiBus>` would be
redundant — `ImuDriver: SpiBus` already elaborates the `SpiBus` bound
wherever `T: ImuDriver` is required, so driver code generic over the
higher-level trait gets the bus-level trait's methods for free.
