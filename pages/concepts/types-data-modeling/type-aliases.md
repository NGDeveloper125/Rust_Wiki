---
title: "Type aliases"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling"]
related_syntax: [type]
see_also: ["The newtype pattern"]
---

## Explanation

A type alias gives an existing type a new name, purely for readability —
it does not create a new, distinct type the way a
[newtype](the-newtype-pattern.md) does. `type Kilometers = f64;` is a
typical example: a value declared as `Kilometers` is directly
interchangeable with one declared as plain `f64`.

Aliases are most valuable for shortening long, repeated type signatures
(especially generic ones, like `type Result<T> = std::result::Result<T, MyError>;`,
a very common pattern for a crate's own error type) and for giving
context to an otherwise-anonymous type in a signature. Because an alias
is fully interchangeable with what it aliases, it provides zero type
safety benefit on its own — if the goal is to prevent two `f64` values
that mean different things from being mixed up, a
[newtype](the-newtype-pattern.md) (an actual distinct type) is the tool
for that, not a type alias.

## Basic usage example

```
type Kilometers = f64; // <- just another name for f64, not a distinct type

let distance: Kilometers = 5.0;
let x: f64 = distance; // <- fine: Kilometers and f64 are the same type
```

**Restriction:** an alias provides no type safety — it's fully
interchangeable with what it aliases, so the compiler won't catch two
aliases of the same underlying type being mixed up (unlike a
[newtype](the-newtype-pattern.md)).

## Best practices & deeper information

### Scenario: Designing a public API

An alias earns its place when a long, repeated generic type would
otherwise clutter every signature it appears in — but it's worth being
honest that it buys readability only, nothing more.

```
use std::collections::HashMap;

type SensorIndex = HashMap<String, Vec<u32>>; // <- pure readability: SensorIndex IS a HashMap, nothing new

fn build_index(entries: &[(String, u32)]) -> SensorIndex { // <- reads far better than the full HashMap<...> type
    let mut index: SensorIndex = HashMap::new();
    for (name, reading) in entries {
        index.entry(name.clone()).or_default().push(*reading);
    }
    index
}

// but the alias gives zero type safety: this still compiles, alias or not --
let raw: HashMap<String, Vec<u32>> = build_index(&[]); // <- SensorIndex and this HashMap are the same type
```

**Why this way:** an alias is fully interchangeable with what it
aliases, so it's the right tool purely for shortening a long,
repeated signature — the moment the goal is preventing two values that
happen to share an underlying type from being mixed up, that's a job for
[the newtype pattern](the-newtype-pattern.md) instead, which actually
creates a new, distinct type.

## Explanation (Embedded)

The mechanism doesn't change at all under `#![no_std]` — a type alias is
purely a compile-time naming convenience with zero runtime representation
of its own, so everything the classic explanation says (including the
"zero type-safety benefit" caveat) applies unchanged. Where it earns an
unusually large amount of its keep in embedded code is HAL types: a
configured peripheral's real type is often a deeply generic signature
spelled out by the HAL crate — a UART configured onto a specific pair of
pins in a specific alternate-function mode might have a real type along
the lines of
`Uart<USART1, (PA9<AlternateFunction7>, PA10<AlternateFunction7>)>`.
Writing that signature out at every function boundary that touches the
peripheral is exactly the "long, repeated generic type" case the classic
explanation calls out — an alias like
`type Uart1 = Uart<USART1, (PA9<AlternateFunction7>, PA10<AlternateFunction7>)>;`
lets application code pass "a `Uart1`" around without re-deriving or
re-typing the HAL's exact generic signature at each call site. (The exact
generic parameters here are illustrative — they vary by HAL crate and
chip family; the pattern of aliasing a verbose configured-peripheral type
is what's genuinely embedded-idiomatic, not this specific signature.)

## Basic usage example (Embedded)

```
// illustrative -- the real generic parameters depend on the HAL crate and chip in use
type Uart1 = Uart<USART1, (PA9<AlternateFunction7>, PA10<AlternateFunction7>)>; // <- one name for a long HAL type

fn send_byte(uart: &mut Uart1, byte: u8) { // <- reads far better than the full generic signature
    uart.write(byte);
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A driver crate built on top of a HAL often wants its public functions to
accept "the configured UART" without forcing every downstream user to
spell out the HAL's full generic instantiation.

```
// illustrative HAL-style generic type -- exact parameters vary by crate/chip
type DebugUart = Uart<USART2, (PA2<AlternateFunction7>, PA3<AlternateFunction7>)>;

pub fn log_line(uart: &mut DebugUart, message: &str) { // <- the alias, not the full generic type, is the public API surface
    for byte in message.bytes() {
        uart.write(byte);
    }
}
```

**Why this way:** the alias buys pure readability, exactly as on hosted
targets — it's worth remembering `DebugUart` is fully interchangeable
with the HAL's real generic type, so it documents intent without adding
any type-level guarantee of its own.

### Scenario: Handling and propagating errors

A crate's own `Result` alias, fixing the error type while leaving the
success type generic, is just as common in `#![no_std]` code as in hosted
Rust — only the underlying `Result` comes from `core` instead of `std`.

```
#![no_std]

type Result<T> = core::result::Result<T, SensorError>; // <- fixes the error type; T stays generic per call site

enum SensorError {
    NotResponding,
    OutOfRange,
}

fn read_temperature() -> Result<i16> { // <- reads as "a Result with this crate's error type", same idiom as std
    Ok(215)
}
```

**Why this way:** aliasing a crate-wide `Result<T>` over a fixed error
type is the same idiom `std`-based crates use for their own error types —
it saves repeating the error type at every fallible function's
signature, and works identically against `core::result::Result` since the
alias mechanism itself has no `std` dependency.
