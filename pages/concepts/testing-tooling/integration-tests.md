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

## Embedded Rust Notes

**Partial support.** Like unit tests, the `tests/` harness links against
`std` and runs as an ordinary host binary, so it cannot exercise code that
only runs correctly on the target hardware. Embedded crates that expose a
hardware-independent public API (a protocol encoder, a configuration
parser) can and do use `tests/` normally, compiled for the host; anything
that needs the real peripherals is instead covered by an on-target,
hardware-in-the-loop test framework such as `defmt-test`, run on real
silicon rather than through `cargo test`.
