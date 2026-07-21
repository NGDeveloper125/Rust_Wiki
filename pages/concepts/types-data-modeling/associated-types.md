---
title: "Associated types"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Writing Generic & Reusable Code", "Generic Programming"]
related_syntax: []
see_also: ["Generics", "Traits", "The Iterator trait"]
---

## Explanation

An associated type is a type placeholder attached to a trait, filled in
by each specific implementation rather than chosen by the caller — for
instance, the standard `Iterator` trait declares `type Item;` and returns
`Option<Self::Item>` from `next`, without fixing what `Item` actually is
until a concrete type implements it.

Every type implementing `Iterator` picks exactly one concrete `Item` type
— the by-value iterator from `Vec<i32>` has `Item = i32`, and
`HashMap<K, V>`'s has `Item = (K, V)` (the borrowing `.iter()` yields
`&i32` and `(&K, &V)` respectively, each its own iterator type) — and that
choice is fixed for that implementation, unlike a generic type parameter,
which a caller could instantiate differently at each use site.

The distinction matters: if `Iterator` used a generic parameter instead
(`trait Iterator<Item> { ... }`), a single type could implement `Iterator<i32>`
*and* `Iterator<String>` simultaneously, which is rarely what you want for
something like "the type this iterator yields" — there's naturally
exactly one right answer per implementing type. Associated types express
that "exactly one, determined by the implementer" relationship directly,
while a generic parameter would leave it open to the caller in a way that
doesn't fit the intent.

## Basic usage example

```
trait Container {
    type Item;                 // <- associated type: each implementer fills this in
    fn get(&self, i: usize) -> Self::Item;
}

struct Numbers(Vec<i32>);

impl Container for Numbers {
    type Item = i32;           // <- this impl fixes Item to i32, exactly once
    fn get(&self, i: usize) -> i32 { self.0[i] }
}
```

## Best practices & deeper information

### Scenario: Implementing traits

When a trait method's result type has exactly one right answer per
implementer, fixing it as an associated type keeps the trait's signature
simple and keeps callers from having to specify a type parameter that was
never theirs to choose.

```
trait Parser {
    type Output;                     // <- fixed per implementer, not chosen by the caller
    fn parse(&self, input: &str) -> Self::Output;
}

struct IntParser;
impl Parser for IntParser {
    type Output = i32;               // <- IntParser always produces i32, never anything else
    fn parse(&self, input: &str) -> i32 {
        input.parse().unwrap_or(0)
    }
}

// A generic `trait Parser<Output> { ... }` would instead let one type implement
// Parser<i32> AND Parser<String> at once -- rarely the right shape for
// "the type this parser produces," which should have exactly one answer.
```

**Why this way:** use an associated type when there's exactly one correct
answer per implementer (an iterator's `Item`, a parser's `Output`);
reach for a generic parameter instead when a caller legitimately needs to
choose it at the call site, the way a
[trait bound](../traits-polymorphism/trait-bounds.md) like `From<T>` lets
one type convert from many different `T`s.

## Explanation (Embedded)

Associated types are a pure compile-time mechanism with no `std` or
allocator dependency, and the `embedded-hal` ecosystem leans on them
specifically to solve a problem generics alone don't: every peripheral
trait (`I2c`, `SpiBus`, `OutputPin`, …) declares an associated
`type Error;` rather than fixing one concrete error type, because each
vendor's HAL implementation has its own genuinely different error
representation — one MCU's I2C peripheral might fail with a bus-specific
NACK/arbitration-loss enum, another's with a different set of cases
entirely. There is exactly one right `Error` type per concrete
implementation, the same "one true answer per implementer" relationship
the classic Explanation describes for `Iterator::Item` — which is why
this is an associated type on the trait rather than a generic parameter
a caller would otherwise have to plumb through every generic driver
function that touches the bus.

This is what lets a driver crate written generically over
`BUS: embedded_hal::i2c::I2c` (see
[Generics' embedded section](generics.md) for that pattern in full)
propagate whatever error type the concrete HAL underneath actually
produces, via an ordinary `?`, without `embedded-hal` needing to invent
one universal error enum that would have to somehow cover every vendor's
hardware-specific failure modes — which isn't really possible, since
those failure modes genuinely differ by silicon.

## Basic usage example (Embedded)

```
trait I2cBus {
    type Error; // <- fixed per concrete bus implementation, not chosen by the driver calling it

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error>;
}

struct Stm32I2c;
enum Stm32I2cError { Nack, ArbitrationLoss }

impl I2cBus for Stm32I2c {
    type Error = Stm32I2cError; // <- this vendor's HAL fixes its own concrete error type, once
    fn write(&mut self, _addr: u8, _bytes: &[u8]) -> Result<(), Stm32I2cError> {
        Ok(())
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Implementing traits

A sensor driver generic over an I2C bus trait should propagate the bus's
own associated `Error` type rather than committing to one error type that
would have to be shared across every vendor's incompatible HAL.

```
trait I2cBus {
    type Error;
    fn write_read(&mut self, addr: u8, out: &[u8], in_: &mut [u8]) -> Result<(), Self::Error>;
}

struct TempSensor<BUS: I2cBus> {
    bus: BUS,
    address: u8,
}

impl<BUS: I2cBus> TempSensor<BUS> {
    fn read_raw(&mut self) -> Result<u16, BUS::Error> { // <- BUS::Error: whatever this specific HAL defines
        let mut raw = [0u8; 2];
        self.bus.write_read(self.address, &[0x00], &mut raw)?;
        Ok(u16::from_be_bytes(raw))
    }
}
```

**Why this way:** `BUS::Error` lets `TempSensor` compile against any
HAL implementing `I2cBus`, each with its own genuinely different error
representation, without the driver crate having to pick — or the
`embedded-hal` ecosystem having to standardize — a single error enum
broad enough to cover every vendor's distinct failure modes.

### Scenario: Handling and propagating errors

A driver function several layers removed from the bus itself should be
able to propagate a bus error with a plain `?`, letting the concrete
error type flow all the way up to whichever caller actually knows how to
handle that specific HAL's failures.

```
trait I2cBus {
    type Error;
    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error>;
}

struct Display<BUS: I2cBus> { bus: BUS, address: u8 }

impl<BUS: I2cBus> Display<BUS> {
    fn set_brightness(&mut self, level: u8) -> Result<(), BUS::Error> {
        self.bus.write(self.address, &[0x81, level])?; // <- BUS::Error propagates untouched through `?`
        Ok(())
    }

    fn clear(&mut self) -> Result<(), BUS::Error> {
        self.bus.write(self.address, &[0x01])?;
        Ok(())
    }
}
```

**Why this way:** because `Self::Error` is fixed per bus implementation
rather than erased or boxed away, a caller several layers up still gets
the exact concrete error variant the underlying HAL produced — useful on
targets where matching on a specific bus fault (NACK vs. timeout vs.
arbitration loss) drives a different recovery action, information a
one-size-fits-all boxed error type would have thrown away.
