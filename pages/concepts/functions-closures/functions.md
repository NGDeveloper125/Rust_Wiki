---
title: "Functions"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures"]
related_syntax: [fn, "->", return, "( )"]
see_also: ["Expression-oriented language", "Closures & capturing", "Higher-order functions"]
---

## Explanation

A function is a named, reusable block of code: it takes zero or more typed
parameters, runs a body of statements and expressions, and produces a
value of a declared return type. Once written, a function can be called
from anywhere it's in scope, as many times as needed, without repeating
its body — the most basic unit of abstraction and code reuse in Rust, and
in almost every programming language before it.

Functions exist so that a piece of logic can be named, tested, and
changed in one place instead of copied wherever it's needed. In Rust
specifically, a function's signature is also a contract: every parameter
has an explicit type (never inferred from how it's called, unlike a
closure's parameters — see [Closures & capturing](closures-and-capturing.md)),
and the return type is stated up front, so both the compiler and the
reader know exactly what a function expects and promises without reading
its body.

A function's body follows Rust's [expression-oriented](expression-oriented-language.md)
rules: the final expression, written without a trailing semicolon, is the
value the function returns. `return` is only needed to exit early, from
somewhere other than the last line — this is why so many idiomatic Rust
functions have no explicit `return` statement at all.

Functions are also the foundation everything else in this group builds
on. A closure is essentially a function value that can additionally
capture variables from its surrounding scope; a
[higher-order function](higher-order-functions.md) is simply a function
that takes or returns another function (or closure); and a
[function pointer](function-pointers.md) is the plain, non-capturing type
a function's name itself has, distinct from a closure's type. Understanding
plain functions first is what makes each of those extensions make sense as
"a function, plus one specific capability," rather than unrelated new
syntax.

## Basic usage example

```
fn celsius_to_fahrenheit(celsius: f64) -> f64 { // <- a function: name, typed parameters, declared return type
    celsius * 9.0 / 5.0 + 32.0
}

let boiling = celsius_to_fahrenheit(100.0);
```

## Best practices & deeper information

### Scenario: Writing generic code

A function that clamps a value into a valid range is useful for any
orderable type — a sensor reading, a retry count, a percentage — so it's
written once, generic over `T`, instead of once per concrete type.

```
fn clamp_reading<T: PartialOrd>(value: T, min: T, max: T) -> T {
    // <- one function definition, generic over `T`, works for any orderable type
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

let capped_temp = clamp_reading(105.2, -40.0, 85.0);
let capped_retries = clamp_reading(12_i32, 0, 10);
```

**Why this way:** a generic function is monomorphized per concrete type
it's called with, so this costs nothing at runtime compared to hand-writing
`clamp_temp` and `clamp_retries` separately — the
[Book's generics chapter](https://doc.rust-lang.org/book/ch10-01-syntax.html)
covers the bound syntax used here.

### Scenario: Handling and propagating errors

Parsing a port number out of a configuration string can fail, so the
function that does the parsing declares `Result` as its return type
instead of panicking or returning a sentinel value like `0`.

```
fn parse_port(raw: &str) -> Result<u16, std::num::ParseIntError> {
    // <- the function's signature makes failure part of its contract
    raw.trim().parse()
}

fn start_server(raw_port: &str) -> Result<(), std::num::ParseIntError> {
    let port = parse_port(raw_port)?;
    println!("listening on port {port}");
    Ok(())
}
```

**Why this way:** putting `Result` in the return type makes failure
visible to every caller at compile time instead of relying on
documentation or a runtime panic, the idiom the
[Book's error-handling chapter](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
builds around.

### Scenario: Designing a public API

A function's parameter types are part of its API surface: accepting a
borrow instead of an owned value lets callers keep using their data
after the call, and returning an owned value keeps the function's
result independent of the caller's data.

```
pub struct Order {
    pub total_cents: u64,
}

pub fn total_with_tax(order: &Order, tax_rate: f64) -> u64 {
    // <- takes a borrow (caller keeps `order`), returns an owned u64
    (order.total_cents as f64 * (1.0 + tax_rate)).round() as u64
}
```

**Why this way:** favoring borrowed parameters over owned ones unless
ownership is genuinely needed gives callers the most flexibility, per the
[API Guidelines' flexibility checklist](https://rust-lang.github.io/api-guidelines/flexibility.html).

## Explanation (Embedded)

Functions are a core-language construct with zero built-in runtime
dependency, so everything about them — named parameters with explicit
types, a declared return type, monomorphization for generics — is
identical whether the target is a server or a microcontroller with no
operating system at all. The place this shows up constantly in embedded
code is HAL and driver method signatures: a peripheral driver's methods
look exactly like any other Rust function, e.g. a register-write method
that borrows `&mut self` (exclusive access to the peripheral), takes a
register address and a value, and returns a `Result` so a bus failure is
part of the function's contract rather than a panic.

Embedded code also has two special-purpose kinds of function that are
still, syntactically and semantically, ordinary functions: the
reset/entry-point function a `#![no_main]` firmware crate supplies
(typically via `cortex-m-rt`'s `#[entry]`, itself a
`#[no_mangle] extern "C" fn` under the hood) and interrupt handlers
registered into a vector table. Neither needs new syntax to exist — see
[`#[no_main]`](../../syntax/attributes/no-main-attribute.md) for how a
firmware crate's entry point is wired up; this page's Explanation focuses
on the ordinary driver-method case, since the entry-point angle is
already covered there.

## Basic usage example (Embedded)

```
struct Error;

struct Bus;

impl Bus {
    fn write_register(&mut self, addr: u8, value: u8) -> Result<(), Error> {
        // <- ordinary function: typed parameters, declared return type, borrows `&mut self`
        Ok(())
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A driver's register-write method borrows `&mut self` (since writing a
register is a mutating operation on shared hardware state) and takes the
address and value as plain typed parameters, rather than bundling them
into a struct or taking ownership of the bus.

```
struct Error;

struct I2cBus;

impl I2cBus {
    fn write_register(&mut self, addr: u8, value: u8) -> Result<(), Error> {
        // <- `&mut self`: exclusive access to the bus for the duration of the write
        // pretend this issues the actual I2C transaction
        Ok(())
    }
}

fn configure_sensor(bus: &mut I2cBus) -> Result<(), Error> {
    bus.write_register(0x20, 0x07)?;
    bus.write_register(0x21, 0x01)
}
```

**Why this way:** borrowing `&mut self` instead of consuming the bus by
value lets a caller keep using it for the next register write, the same
borrow-first guidance the
[API Guidelines' flexibility checklist](https://rust-lang.github.io/api-guidelines/flexibility.html)
gives for any method whose receiver the caller needs again.

### Scenario: Handling and propagating errors

A register write over a real bus (I2C, SPI) can fail — a NACK, a bus
timeout — so the driver function's signature declares that failure
explicitly with `Result`, instead of panicking or silently ignoring a
failed transaction.

```
enum BusError {
    Nack,
    Timeout,
}

fn write_register(addr: u8, value: u8) -> Result<(), BusError> {
    // <- the signature makes bus failure part of the function's contract, not a surprise panic
    if addr > 0x7F {
        return Err(BusError::Nack);
    }
    Ok(())
}

fn init_device() -> Result<(), BusError> {
    write_register(0x00, 0x01)?;
    write_register(0x01, 0x00)
}
```

**Why this way:** a firmware panic on a failed bus transaction typically
halts the whole device with no operator present to see a message, so
returning `Result` and propagating with `?` lets calling code retry, fall
back, or report the failure through whatever channel the application has
— the same
[Book error-handling](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
discipline, applied to a failure mode that's routine on real hardware.
