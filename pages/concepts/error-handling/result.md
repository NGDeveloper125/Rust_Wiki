---
title: "Result<T, E>"
area: "Error Handling"
embedded_support: full
groups: ["Error Handling", "Functional Programming", "Handling Errors & Failure", "Unique to Rust", "Coming from Python / JavaScript", "Coming from Haskell / functional languages"]
related_syntax: [match, "if let", "?"]
see_also: ["Option<T>", "The ? operator (concept angle)", "Custom error types", "The Error trait"]
---

## Explanation

`Result<T, E>` is Rust's type for an operation that might not succeed,
and needs to say why when it doesn't. Like [`Option<T>`](option.md) it's
an ordinary two-variant enum — `Ok(T)` holding the success value, `Err(E)`
holding the failure — but where `Option` only records that something is
missing, `Result` carries a full value describing *what went wrong*, so a
caller can log it, match on its kind, or wrap it with more context.

It exists to replace exceptions. Languages built around `try`/`catch` let
a function throw a value whose type doesn't appear anywhere in its
signature, so nothing at compile time tells a caller which calls can fail
or what they can fail with. Rust makes failure part of the type: a
function that can fail returns `Result<T, E>`, so the compiler forces
every caller to at least acknowledge the possibility, whether by
matching, propagating with [`?`](the-question-mark-operator.md), or
explicitly choosing to `unwrap()` and accept a [panic](panic-and-unwinding.md).

The mental model is a fork in the road with data riding on both branches:
every `Result`-returning call produces exactly one of two outcomes, and
code downstream literally cannot reach the success value without going
through that fork. This is stricter than "hope the exception doesn't
happen" — there's no path through the code that silently skips handling
failure.

`Result` composes rather than being handled one match arm at a time.
Combinators like `.map()` (transform the `Ok` value), `.map_err()`
(transform the `Err` value, often to adapt one layer's error type into
another's — see [custom error types](custom-error-types.md)),
`.and_then()` (chain a further fallible step), and `.unwrap_or()` (supply
a fallback) let a pipeline of fallible steps read linearly instead of as
nested matches. `?` takes this composition further, turning an early
return into a single character at each fallible step.

What `E` should actually be is a design decision in its own right: a
quick script might get away with `Result<T, String>`, but a well-designed
library defines its own error enum (see
[custom error types](custom-error-types.md)) that implements the standard
[`Error` trait](the-error-trait.md), so callers get a stable, matchable,
composable failure type instead of an opaque message.

## Basic usage example

```
fn parse_reading(raw: &str) -> Result<f64, std::num::ParseFloatError> { // <- explicit "maybe failed" return type
    raw.trim().parse()
}

match parse_reading("21.5") {
    Ok(value) => println!("reading: {value}"),
    Err(e) => println!("invalid reading: {e}"),
}
```

## Best practices & deeper information

### Scenario: Handling and propagating errors

A sensor driver's raw reading can fail to parse; the caller turns that
low-level error into a domain-specific one with `map_err` rather than
exposing `ParseFloatError` to its own callers.

```
#[derive(Debug)]
struct SensorError(String);

fn read_temperature(raw: &str) -> Result<f64, SensorError> {
    raw.trim()
        .parse::<f64>()
        .map_err(|e| SensorError(format!("bad reading {raw:?}: {e}"))) // <- Result::map_err adapts the error type
}

fn average(raws: &[&str]) -> Result<f64, SensorError> {
    let mut total = 0.0;
    for raw in raws {
        total += read_temperature(raw)?;
    }
    Ok(total / raws.len() as f64)
}
```

**Why this way:** converting a lower-level error into the caller's own
domain error at the boundary keeps failure information relevant to
callers instead of leaking implementation details, an approach the
[Rust Book's error handling chapter](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
and the API Guidelines both recommend at library boundaries.

### Scenario: Validating input

Constructing an `Order` should be impossible with an invalid quantity, so
the constructor returns `Result<Order, OrderError>` instead of silently
clamping or panicking.

```
#[derive(Debug, PartialEq)]
enum OrderError {
    ZeroQuantity,
}

struct Order {
    sku: String,
    quantity: u32,
}

impl Order {
    fn new(sku: &str, quantity: u32) -> Result<Order, OrderError> { // <- fallible constructor: invalid orders can't exist
        if quantity == 0 {
            return Err(OrderError::ZeroQuantity);
        }
        Ok(Order { sku: sku.to_string(), quantity })
    }
}

let result = Order::new("SKU-1", 0);
assert!(matches!(result, Err(OrderError::ZeroQuantity))); // <- Result<Order, OrderError> makes failure explicit
```

**Why this way:** returning `Result` from the constructor makes "quantity
must be nonzero" a compile-time-checked part of the type's contract
instead of a comment, applying the parse-don't-validate idiom from
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/) and
[Effective Rust](https://effective-rust.com/).

### Scenario: Testing

A unit test checks that invalid input produces the expected `Err`
variant, not just that the function "didn't crash."

```
fn divide_evenly(total: u32, parts: u32) -> Result<u32, String> {
    if parts == 0 {
        return Err("cannot divide by zero parts".to_string());
    }
    if total % parts != 0 {
        return Err(format!("{total} does not divide evenly into {parts} parts"));
    }
    Ok(total / parts)
}

#[test]
fn rejects_uneven_split() {
    let outcome = divide_evenly(10, 3); // <- Result lets the test assert on *why* it failed
    assert!(outcome.is_err());
    assert_eq!(outcome.unwrap_err(), "10 does not divide evenly into 3 parts");
}
```

**Why this way:** asserting on the specific `Err` payload, not just that
the call failed, catches regressions where the function still fails but
for the wrong reason or with a broken message — consistent with the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
on writing assertions that check specific outcomes.

## Explanation (Embedded)

`Result<T, E>` is `core::result::Result` — the same two-variant enum, with
identical semantics and no allocator required by the type itself; only the
choice of `E` (a `String`-based error, say, which needs `alloc`) can pull
in a dependency the type itself doesn't demand.

The design weight is higher in embedded code, though, not lower. On a
hosted program there's usually a person or a log aggregator watching
stderr, so an occasionally-swallowed error is merely sloppy. On a deployed
device there is often nobody watching anything: `Result<(), E>` coming back
from an SPI transaction, an I2C read, or a flash write is frequently the
*only* signal that something went wrong at all. `embedded_hal`'s traits
(`I2c`, `SpiDevice`, and the rest) return `Result<T, Self::Error>` from
every fallible operation for exactly this reason, and consistently
propagating that `Result` — logging it over UART/RTT, retrying, or falling
back to a known-safe state — rather than reaching for `.unwrap()` or
discarding it with `.ok()`, is what actually surfaces a hardware fault
instead of letting it disappear silently into a wrong reading or a hung
peripheral.

## Basic usage example (Embedded)

```
use embedded_hal::i2c::I2c;

fn read_register<I: I2c>(i2c: &mut I, addr: u8, reg: u8) -> Result<u8, I::Error> { // <- explicit "may fail" return type
    let mut buf = [0u8; 1];
    i2c.write_read(addr, &[reg], &mut buf)?;
    Ok(buf[0])
}
```

## Best practices & deeper information (Embedded)

### Scenario: Handling and propagating errors

A sensor driver turns a low-level SPI transfer failure into its own error
type with `map_err`, then an averaging function propagates that same error
across three chained reads with `?` instead of unwrapping past a bus
fault.

```
use embedded_hal::spi::SpiDevice;

#[derive(Debug)]
struct SensorError;

fn read_raw(spi: &mut impl SpiDevice, cmd: u8) -> Result<u16, SensorError> {
    let mut buf = [0u8; 2];
    spi.transfer(&mut buf, &[cmd]).map_err(|_| SensorError)?; // <- Result::map_err adapts the low-level SPI error
    Ok(u16::from_be_bytes(buf))
}

fn average_of_three(spi: &mut impl SpiDevice, cmd: u8) -> Result<u32, SensorError> {
    let mut total: u32 = 0;
    for _ in 0..3 {
        total += read_raw(spi, cmd)? as u32; // <- propagates instead of unwrapping past a bus fault
    }
    Ok(total / 3)
}
```

**Why this way:** propagating the `Result` instead of swallowing it with
`.ok()` or panicking on it matters more here than on a hosted program — a
bus fault surfaced only as a subtly wrong average is a far harder field bug
to diagnose than one that halts the call chain immediately at the failing
transaction.

### Scenario: Validating input

Writing a duty-cycle percentage to a PWM peripheral rejects an
out-of-range value up front, returning `Result` instead of silently
clamping it to something the caller never asked for.

```
#[derive(Debug, PartialEq)]
struct OutOfRange;

fn set_duty_cycle(percent: u8) -> Result<(), OutOfRange> {
    if percent > 100 {
        return Err(OutOfRange);
    }
    // ... write `percent` into the timer's compare register
    Ok(())
}

assert_eq!(set_duty_cycle(150), Err(OutOfRange));
```

**Why this way:** rejecting the bad value explicitly, rather than clamping
it silently to 100%, keeps a caller's logic bug from masquerading as
normal operation — a real risk on a device where the wrong duty cycle may
run unnoticed for a long time before anyone checks the output.
