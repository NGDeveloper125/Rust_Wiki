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

## Embedded Rust Notes

**Partial support.** The `#[test]` harness itself needs `std` — it
depends on catching unwinding panics and printing a pass/fail report,
neither of which exists on bare metal. In practice this rarely stops
embedded projects from unit testing: hardware-independent logic (parsing,
math, protocol encoding) is tested the ordinary way with `cargo test`
running on the *host* development machine, compiled without the target's
`#![no_std]` restriction. Only code that genuinely touches hardware needs
a different approach — an on-target framework like `defmt-test` runs
`#[test]`-like functions on the real microcontroller and reports results
over the debug probe instead of through the host test harness.
