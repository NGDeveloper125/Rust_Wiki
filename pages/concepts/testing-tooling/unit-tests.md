---
title: "Unit tests"
area: "Testing & Tooling"
embedded_support: partial
groups: ["Testing & Tooling", "Testing & Documenting Code", "Testing"]
related_syntax: ["#[test]", "#[cfg(...)]"]
see_also: ["Integration tests", "Doc tests", "Benchmarking", "Traits"]
---

## Explanation

A unit test is a small, self-contained check of a single function or a
narrow piece of behavior, written as an ordinary function marked
`#[test]` and run by `cargo test`. Rust's convention is to keep unit
tests in the same file as the code they exercise, inside a `mod tests`
block annotated `#[cfg(test)]` — a compilation gate that excludes the
whole module from normal `cargo build`/`cargo run` and includes it only
when compiling for `cargo test`. Because the test module lives right
next to the code, it can `use super::*;` to reach private items directly,
which is the defining difference from an [integration test](integration-tests.md):
a unit test checks a module from the inside, including things that never
become part of the crate's public API.

A test function passes as long as it returns normally; it fails the
moment it panics. This is why the `assert!`, `assert_eq!`, and
`assert_ne!` macros are the primary tool inside a test body — each one
panics with a descriptive message on failure, which `cargo test` reports
per-test alongside a pass/fail summary. `cargo test` runs every discovered
`#[test]` function, by default in parallel across threads, so unit tests
should be independent of each other and of any shared external state.

Unit tests are the fastest feedback loop available: no network, no
filesystem, no process boundary, usually no I/O at all — just a function
call and an assertion, compiled straight into the same crate as the code
under test. When the code being tested depends on something slow,
non-deterministic, or external (the system clock, a network call, a
database), the idiomatic move is to depend on a trait instead of the
concrete thing and substitute a test-only implementation, exactly as
described on the [Traits](../traits-polymorphism/traits.md) page's testing
scenario — that keeps unit tests fast and deterministic without any
conditional compilation in the production code path.

Unit tests, [integration tests](integration-tests.md), and
[doc tests](doc-tests.md) together make up what `cargo test` runs; they
differ in *where* the test lives and *what surface* it exercises, not in
how a pass/fail is decided.

## Basic usage example

```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)] // <- only compiled when running `cargo test`
mod tests {
    use super::*;

    #[test] // <- marks this function as a unit test
    fn adds_two_numbers() {
        assert_eq!(add(2, 3), 5);
    }
}
```

## Best practices & deeper information

### Scenario: Testing

A table-driven test — one loop over a list of input/expected pairs —
covers many cases of a pricing calculation without writing a near-
identical `#[test]` function for each one.

```
fn discounted_total(price_cents: u32, percent_off: u8) -> u32 {
    price_cents - (price_cents * percent_off as u32 / 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] // <- one test function, many cases exercised inside it
    fn applies_discount_for_several_cases() {
        let cases = [
            (1000, 0, 1000),
            (1000, 10, 900),
            (2500, 50, 1250),
        ];

        for (price_cents, percent_off, expected) in cases {
            assert_eq!(discounted_total(price_cents, percent_off), expected);
        }
    }
}
```

**Why this way:** a table of cases keeps each scenario readable as data
rather than duplicated control flow, and adding a new case is a one-line
change — the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
builds its own examples the same way, one assertion per meaningfully
different input.

### Scenario: Validating input

A constructor that returns `Result` instead of panicking needs a unit
test on both branches — the accepted case and the rejected case — since
either one regressing silently would be a real bug.

```
struct OrderId(u32);

impl OrderId {
    fn new(value: u32) -> Result<Self, &'static str> {
        if value == 0 {
            Err("order id must be non-zero")
        } else {
            Ok(OrderId(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] // <- covers the accepted path
    fn accepts_a_positive_id() {
        assert!(OrderId::new(42).is_ok());
    }

    #[test] // <- covers the rejected path
    fn rejects_zero() {
        assert!(OrderId::new(0).is_err());
    }
}
```

**Why this way:** testing only the happy path leaves the validation logic
itself unverified — a parse-don't-validate constructor is only as
trustworthy as the tests confirming it actually rejects what it claims to
reject, per the
[Rust API Guidelines' predictability guidance](https://rust-lang.github.io/api-guidelines/predictability.html)
on constructors.

### Scenario: Handling and propagating errors

`#[should_panic]` lets a unit test assert that a function panics on an
invalid input, which is the right tool when the function's contract is
"this is a programmer error, not a recoverable one" rather than a
`Result`-returning failure.

```
fn split_evenly(total_cents: u32, shares: u32) -> u32 {
    if shares == 0 {
        panic!("cannot split into zero shares");
    }
    total_cents / shares
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_total_across_shares() {
        assert_eq!(split_evenly(1000, 4), 250);
    }

    #[test]
    #[should_panic(expected = "cannot split into zero shares")] // <- asserts the panic, and its message
    fn panics_on_zero_shares() {
        split_evenly(1000, 0);
    }
}
```

**Why this way:** pairing `#[should_panic]` with `expected = "..."` checks
that the function panicked *for the intended reason*, not merely that it
panicked somehow — the
[Rust Book](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-for-panics-with-should_panic)
recommends the `expected` argument for exactly this precision.

## Explanation (Embedded)

The core story for unit-testing embedded code is a split by *where the
logic actually lives*, not a single answer. The ordinary `#[test]`
harness needs `std` — it schedules each test on its own OS thread and
catches an unwinding panic to report pass/fail from a hosted process —
none of which a bare `#![no_std]` target provides, so compiling the
harness itself for the microcontroller isn't how this is normally done.

In practice, most of a crate's genuinely unit-testable logic — parsing a
sensor frame, encoding/decoding a protocol message, a calibration
calculation, a checksum — doesn't touch a peripheral at all. That code is
written to also compile for the host (kept on `core`, with `std` pulled
in only behind `#[cfg(test)]`), and is tested with a completely ordinary
`cargo test` run on the development machine, exactly as any other crate.
Only the logic that's genuinely hardware-dependent — a driver's real
register writes, an interrupt handler, timing against a live peripheral
— can't be honestly tested that way, and instead needs `defmt-test` or
`embedded-test` to run the `#[test]`-shaped functions for real on the
target, flashed and executed over a debug probe (`probe-rs`), with
results reported back over RTT rather than printed by a hosted process.

## Basic usage example (Embedded)

```
// This function only manipulates bytes, so it's tested on the host, not the target.
pub fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] // <- runs under an ordinary `cargo test` on the host, no target hardware involved
    fn checksum_of_empty_slice_is_zero() {
        assert_eq!(checksum(&[]), 0);
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Testing

A firmware crate's frame-parsing logic is pure byte manipulation with no
peripheral access, so it belongs in the ordinary host-tested `#[cfg(test)]`
module rather than anywhere near a debug probe.

```
#![no_std]

pub fn parse_frame_length(header: [u8; 2]) -> u16 { // <- hardware-independent: safe to test on the host
    u16::from_be_bytes(header)
}

#[cfg(test)] // <- pulls in `std` for the test build only; the crate itself stays `no_std`
mod tests {
    use super::*;

    #[test]
    fn parses_a_two_byte_length_header() {
        assert_eq!(parse_frame_length([0x01, 0x2C]), 0x012C);
    }
}
```

**Why this way:** running `cargo test` on the host is orders of magnitude
faster and needs no hardware at all, so pushing every function that
*can* be host-tested out of the hardware-dependent path keeps the fast
feedback loop as large as possible; the split mirrors the same
host-vs-target reasoning [`#[test]`](../../syntax/attributes/test-attribute.md)'s
Embedded Rust Notes lay out for the attribute itself.

### Scenario: Validating input

A driver rejects an out-of-range channel selector before touching the
ADC peripheral at all — a check that's still pure logic, so it doesn't
need real hardware to verify even though the type it guards is only ever
constructed right before a register write.

```
#![no_std]

pub struct ChannelId(u8);

impl ChannelId {
    pub fn new(value: u8) -> Result<Self, &'static str> {
        if value > 7 {
            Err("ADC channel out of range (0..=7)")
        } else {
            Ok(ChannelId(value)) // <- validated before any register access happens
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_out_of_range_channel() {
        assert!(ChannelId::new(9).is_err());
    }
}
```

**Why this way:** the validation itself never reads a register, so
testing it on the host is just as trustworthy as testing it on target
and far cheaper to run on every commit; only the eventual register write
`ChannelId` guards needs a `defmt-test`/`embedded-test` on-target test,
and only once real hardware behavior — not input validation — is what's
being verified.
