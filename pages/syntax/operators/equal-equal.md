---
title: "=="
kind: operator
embedded_support: full
groups: [Comparison, Basics]
related_concepts: [Operator overloading, "Derivable traits (Debug, Clone, PartialEq, …)"]
related_syntax: ["!="]
see_also: ["!="]
---

## Explanation

`==` tests equality, overloadable via `std::cmp::PartialEq` (usually
obtained via `#[derive(PartialEq)]` rather than hand-written).

`PartialEq` is "partial" because equality need not be total — floating
point `NaN != NaN`, which is why `f32`/`f64` implement `PartialEq` but not
the stricter `Eq` (which additionally requires reflexivity: `x == x`
always). Comparing two values whose type doesn't implement `PartialEq` is
a compile error, not a runtime failure — there's no default "compare by
reference identity" fallback the way some languages have.

## Usage examples

### Comparing two values for equality

```
let a = 5;
let b = 5;
let same = a == b; // <- `==` compares `a` and `b` for equality
```

**Restriction:** `==` can't be chained — `a == b == c` doesn't compile.
Rust's grammar rejects chained comparison operators outright (rustc
reports "comparison operators cannot be chained" and suggests
`a == b && b == c`); the expression never even reaches trait resolution.

### Validating input

Rejecting a request whose version doesn't match what the server supports
is a plain equality check — `==` compares the field directly rather than
inspecting it piecemeal.

```
struct Request {
    api_version: u32,
    payload: String,
}

const SUPPORTED_VERSION: u32 = 3;

fn is_valid(request: &Request) -> bool {
    request.api_version == SUPPORTED_VERSION // <- `==` checks the request matches what this server supports
}
```

Deriving `PartialEq` on the surrounding types (rather
than hand-rolling comparisons field by field) keeps `==` checks like this
correct as fields are added — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits)
recommend eagerly implementing `PartialEq` wherever structural equality
is the intended comparison.

### Testing

`assert_eq!` uses `==`/`PartialEq` under the hood, but — unlike a bare
`assert!(a == b)` — prints both values on failure, which is why it's the
default choice for equality checks in tests.

```
fn total_price(quantity: u32, unit_price: f64) -> f64 {
    quantity as f64 * unit_price
}

#[test]
fn computes_total_price() {
    let total = total_price(3, 2.5);
    assert_eq!(total, 7.5); // <- `assert_eq!` compares with `==` and reports both sides on failure
}
```

The [Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
recommends `assert_eq!`/`assert_ne!` over a bare `assert!(a == b)`
specifically because the macro captures and prints both operands when
the assertion fails, saving a debugging round trip.

## Explanation (Embedded)

`==` means exactly the same thing under `#![no_std]` — `PartialEq` lives in
`core::cmp`, so nothing about the trait or the comparison changes on a
microcontroller target. What genuinely differs is *what* gets compared:
embedded code constantly checks a hardware identity or status register
against a constant the datasheet defines, rather than comparing two
ordinary in-memory values. A "WHO_AM_I"-style identity register — many
I2C/SPI sensors expose one — exists specifically so driver code can
confirm it's actually talking to the chip it expects before trusting
anything else that chip reports.

## Usage examples (Embedded)

### Confirming a sensor's identity register before use

```
const EXPECTED_WHO_AM_I: u8 = 0x68; // datasheet-defined identity byte for this sensor model

fn is_expected_sensor(who_am_i_reg: u8) -> bool {
    who_am_i_reg == EXPECTED_WHO_AM_I // <- `==` confirms the chip on the bus is the one this driver expects
}

let who_am_i_reg = 0x68; // value read back from the sensor's WHO_AM_I register
assert!(is_expected_sensor(who_am_i_reg));
```

### Checking a peripheral's status flag against an expected encoding

```
const STATUS_READY: u8 = 0x01;

fn is_ready(status_reg: u8) -> bool {
    status_reg == STATUS_READY // <- `==` checks the status register matches the "ready" encoding
}
```
