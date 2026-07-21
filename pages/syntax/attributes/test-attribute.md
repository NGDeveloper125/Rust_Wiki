---
title: "#[test]"
kind: attribute
embedded_support: partial
groups: ["Testing", "Testing & Tooling"]
related_concepts: ["Unit tests", "Doc tests"]
related_syntax: ["#[cfg(...)]", "#[ignore]", "#[should_panic]"]
see_also: ["Unit tests", "#[ignore]", "#[should_panic]"]
---

## Explanation

`#[test]` is placed directly above a function to mark it as a test case,
discovered and run by `cargo test`. Under an ordinary `cargo build` or
`cargo run`, `#[test]`-marked functions don't exist in the compiled
output at all — the test harness itself is only assembled, and
`#[test]`-marked functions only compiled in, when building under the
implicit `#[cfg(test)]` configuration `cargo test` sets, which is why
production binaries carry no trace of test code or its dependencies.
Conventionally a test function sits inside a `mod tests` block explicitly
marked `#[cfg(test)]`, but that module wrapper is a convention, not a
requirement of `#[test]` itself — a `#[test]` function compiles anywhere
`cfg(test)` is active, module or not.

**Signature requirements.** A `#[test]` function takes **no arguments**.
Its return type must be either `()` (the ordinary case — the test passes
by returning normally and fails by panicking, typically via `assert!`,
`assert_eq!`, or `assert_ne!`) or `Result<(), E>` for some error type `E`
implementing `Debug` — returning `Err(...)` fails the test the same way a
panic does, which lets a test use the `?` operator to propagate a
fallible setup step instead of unwrapping it. The mechanics of writing
good unit tests — table-driven cases, testing both the accepted and
rejected paths of a validating constructor, and the tradeoffs against
`#[should_panic]` — are covered in depth on the
[Unit tests](../../concepts/testing-tooling/unit-tests.md) concept page;
this page covers the attribute itself.

`cargo test` runs every discovered `#[test]` function, by default in
parallel across threads and in no guaranteed order, reporting a per-test
pass/fail line followed by a summary. A `#[test]` function can be combined
with [`#[ignore]`](ignore-attribute.md) to skip it by default, or
[`#[should_panic]`](should-panic-attribute.md) to invert what counts as
passing.

## Basic usage example

```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] // <- marks this function as a test case; only compiled under `cargo test`
    fn adds_two_numbers() {
        assert_eq!(add(2, 3), 5);
    }
}
```

## Best practices & deeper information

### Scenario: Testing

A test that needs a fallible setup step — parsing a fixture value, say —
reads more directly with `?` and a `Result`-returning test function than
with a chain of `.unwrap()` calls hiding which step actually failed.

```
fn parse_price_cents(input: &str) -> Result<u32, std::num::ParseIntError> {
    input.trim().parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] // <- Result<(), E> return type: `?` propagates a failure as a failed test, not a panic
    fn parses_a_valid_price() -> Result<(), std::num::ParseIntError> {
        let cents = parse_price_cents("1999")?;
        assert_eq!(cents, 1999);
        Ok(())
    }
}
```

**Why this way:** letting `#[test]` functions return `Result<(), E>` means
a genuinely fallible step in the test's own setup can use `?` instead of
`.unwrap()`, so a setup failure is distinguishable in principle from an
assertion failure further down; the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/testing.html#the-test-attribute)
documents `()` and `Result<(), E>` (with `E: Debug`) as the two accepted
return types for a `#[test]` function.

## Embedded Rust Notes

**Partial support.** The `#[test]` harness depends on `std` — catching
unwinding panics per-test and printing a pass/fail report both assume an
OS underneath. This doesn't stop most embedded logic from being unit
tested in the ordinary way: hardware-independent code (parsing, math,
protocol encoding) is typically tested with `cargo test` running on the
**host** development machine, compiled without the target's `#![no_std]`
restriction, exactly as described in [Unit tests](../../concepts/testing-tooling/unit-tests.md)'s
Embedded Rust Notes. Code that genuinely depends on real hardware needs a
different tool — an on-target framework like `defmt-test` runs
`#[test]`-like functions on the microcontroller itself and reports
results over a debug probe.
