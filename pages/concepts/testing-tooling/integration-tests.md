---
title: "Integration tests"
area: "Testing & Tooling"
embedded_support: partial
groups: ["Testing & Tooling", "Testing & Documenting Code", "Testing"]
related_syntax: ["#[test]"]
see_also: ["Unit tests", "Doc tests", "Crates"]
---

## Explanation

An integration test exercises a crate the same way an outside caller
would: it lives in a top-level `tests/` directory rather than alongside
the implementation, and each file under `tests/` is compiled as its own
separate crate that depends on the library the same way any published
consumer would. That single fact — a real crate boundary, not just a
different module — is why an integration test can only reach `pub`
items. Where a [unit test](unit-tests.md) can see private helpers because
it's compiled inside the same module, an integration test is deliberately
locked out of them, which is exactly the point: it proves the crate
*works when used from outside*, not merely that its internals are
individually correct. The [Crates](../modules-crates-visibility/crates.md)
page covers that crate-boundary mechanics in more depth; this page's
angle is the testing methodology it enables.

That methodology angle matters because a unit test's job is to isolate
one function; an integration test's job is to check that several public
functions still cooperate correctly once wired together — a realistic,
multi-step use of the crate rather than a single call. A suite made
entirely of unit tests can pass in full while the public API is
nevertheless awkward or subtly broken to actually use end-to-end; an
integration test catches exactly that gap, because it has no shortcut
into internals to fall back on.

Shared setup that several integration test files need — building a
sample dataset, spinning up a fixture — commonly lives in
`tests/common/mod.rs` rather than in a file directly under `tests/`: a
plain `mod.rs` isn't itself compiled as a test crate, so it can hold
helper functions without contributing spurious test binaries.

## Basic usage example

```
// src/lib.rs
pub struct Order {
    pub total_cents: u32,
}

pub fn apply_discount(order: &Order, percent_off: u8) -> u32 { // <- part of the crate's public API
    order.total_cents - (order.total_cents * percent_off as u32 / 100)
}

// tests/discount.rs — compiled as its own crate, linked against the library
use order_lib::{Order, apply_discount};

#[test]
fn discount_reduces_total() {
    let order = Order { total_cents: 2000 };
    assert_eq!(apply_discount(&order, 25), 1500);
}
```

## Best practices & deeper information

### Scenario: Testing

A single integration test that walks through several public functions in
sequence checks the *workflow* a real caller would perform, not just each
function in isolation — the thing a collection of unit tests, each
mocking away the next step, cannot catch.

```
// src/lib.rs
pub struct Cart { pub subtotal_cents: u32 }

pub fn new_cart(subtotal_cents: u32) -> Cart {
    Cart { subtotal_cents }
}

pub fn apply_coupon(cart: Cart, percent_off: u8) -> Cart {
    Cart { subtotal_cents: cart.subtotal_cents - (cart.subtotal_cents * percent_off as u32 / 100) }
}

pub fn total_with_tax(cart: &Cart, tax_percent: u8) -> u32 {
    cart.subtotal_cents + (cart.subtotal_cents * tax_percent as u32 / 100)
}

// tests/checkout_flow.rs
use shop::{new_cart, apply_coupon, total_with_tax};

#[test] // <- exercises the public workflow end to end, not one function at a time
fn checkout_applies_coupon_then_tax() {
    let cart = new_cart(10_000);
    let cart = apply_coupon(cart, 10);
    assert_eq!(total_with_tax(&cart, 8), 9_720);
}
```

**Why this way:** chaining real public calls together is what actually
proves the crate's pieces compose the way a caller expects, which the
[Rust Book's chapter on test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests)
frames as integration tests treating the library "exactly the same as any
other code that uses that library."

### Scenario: Handling and propagating errors

Because an integration test only sees what's `pub`, it's also the most
honest place to check that a crate's *public* error type actually reports
failures the way its documentation promises.

```
// src/lib.rs
#[derive(Debug, PartialEq)]
pub struct ConfigError(pub String);

pub fn parse_timeout_seconds(raw: &str) -> Result<u32, ConfigError> {
    raw.parse().map_err(|_| ConfigError(format!("invalid timeout: {raw:?}")))
}

// tests/config_parsing.rs
use app_config::{parse_timeout_seconds, ConfigError};

#[test]
fn rejects_non_numeric_timeout() {
    let result = parse_timeout_seconds("soon");
    assert_eq!(result, Err(ConfigError("invalid timeout: \"soon\"".into())));
}
```

**Why this way:** this only compiles and passes if `ConfigError` and its
fields are actually reachable and usable from outside the crate — an
internal-only error type would fail here even if a same-module unit test
never noticed, matching the
[Rust Book's](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests)
description of integration tests as a check on the crate's usable public
surface.

### Scenario: Designing a public API

Writing the first integration test for a new module is usually the first
time anyone calls its public functions the way an outside crate would —
awkward call sites at that point are a signal to reshape the API before
it ships, not after.

```
// tests/common/mod.rs — shared setup, not itself a test file
pub fn sample_order() -> order_lib::Order {
    order_lib::Order { total_cents: 4599 }
}

// tests/pricing.rs
mod common;
use order_lib::apply_discount;

#[test]
fn applies_discount_to_a_sample_order() { // <- calling the crate exactly like an external user would
    let order = common::sample_order();
    assert_eq!(apply_discount(&order, 10), 4139);
}
```

**Why this way:** an integration test is, in effect, the crate's first
real client — a signature that's painful to call correctly here will be
just as painful for every downstream user, which is why the
[API Guidelines](https://rust-lang.github.io/api-guidelines/) frame a
usable, ergonomic call site as part of the API's design, not an
afterthought to be discovered later from bug reports.

## Explanation (Embedded)

The same host-vs-target split that governs [unit tests](unit-tests.md)
applies here, with one extra wrinkle specific to integration tests: the
classic `tests/` directory convention assumes each file under it can be
compiled and linked as its own `std` test binary that the harness builds
and runs directly. That assumption holds fine for the slice of an
embedded crate's public API that's hardware-independent — a protocol
encoder, a configuration parser — which can sit in `tests/` and run on
the host exactly like any other crate's integration tests. It stops
holding the moment an integration test needs to exercise the crate's
public API *against real hardware*: `tests/` has no mechanism to flash a
binary onto a microcontroller and run it there, so on-target integration
testing isn't done through the ordinary `tests/` directory at all — it
goes through `defmt-test`'s or `embedded-test`'s own harness instead,
which builds a firmware image, flashes it via a debug probe, and reports
each test's pass/fail back over RTT.

## Basic usage example (Embedded)

```
// src/lib.rs
#![no_std]

pub struct Frame { pub id: u8, pub payload: u8 }

pub fn encode_frame(frame: &Frame) -> [u8; 2] { // <- part of the crate's public API, no hardware touched
    [frame.id, frame.payload]
}

// tests/encode.rs — an ordinary host-run integration test; no target hardware needed
use protocol::{Frame, encode_frame};

#[test]
fn encodes_id_and_payload_in_order() {
    let frame = Frame { id: 7, payload: 42 };
    assert_eq!(encode_frame(&frame), [7, 42]);
}
```

## Best practices & deeper information (Embedded)

### Scenario: Testing

A crate's public frame-encoding API is pure logic, so its integration
test lives in the ordinary `tests/` directory and runs on the host —
exactly the workflow [`#[test]`](../../syntax/attributes/test-attribute.md)'s
Embedded Rust Notes describe for hardware-independent code in general.

```
// src/lib.rs
#![no_std]

pub struct Reading { pub channel: u8, pub millivolts: u16 }

pub fn to_wire_bytes(reading: &Reading) -> [u8; 3] {
    let mv = reading.millivolts.to_be_bytes();
    [reading.channel, mv[0], mv[1]]
}

// tests/wire_format.rs
use sensor_proto::{Reading, to_wire_bytes};

#[test] // <- ordinary host-run integration test: `tests/` still applies for hardware-independent APIs
fn encodes_channel_and_millivolts() {
    let reading = Reading { channel: 2, millivolts: 3300 };
    assert_eq!(to_wire_bytes(&reading), [2, 0x0C, 0xE4]);
}
```

**Why this way:** nothing about `to_wire_bytes` depends on real silicon,
so testing it through the ordinary `tests/` mechanism keeps the fast,
no-hardware-needed feedback loop that host-run tests give any other
crate.

### Scenario: Designing a public API

Writing the first test against a driver's public `init`/`read` sequence
is often the first time anyone calls it the way a real firmware would —
but for a driver that only behaves correctly against actual silicon, that
first call has to happen on real hardware, through `embedded-test`'s
harness rather than `tests/`.

```
#![no_std]
#![no_main]

#[embedded_test::tests] // <- on-target integration test: flashed and run via a debug probe, not `tests/`
mod tests {
    use my_hal::TemperatureSensor;

    #[test]
    fn sensor_reports_a_plausible_reading() {
        let mut sensor = TemperatureSensor::init().expect("sensor init failed"); // <- exercises the public API end to end, on real hardware
        let celsius = sensor.read().expect("read failed");
        assert!((-40.0..125.0).contains(&celsius));
    }
}
```

**Why this way:** `TemperatureSensor::init`/`read` only behave correctly
wired to a real sensor over a real bus, so a host-run `tests/` file could
only fake the response, not prove the driver actually works — exercising
the same public sequence through `embedded-test` on the target is what
plays the "first real client" role an integration test is meant to.
