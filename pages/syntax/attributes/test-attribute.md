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

## Usage examples

### Marking a function as a test case

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

### Testing

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

Letting `#[test]` functions return `Result<(), E>` means
a genuinely fallible step in the test's own setup can use `?` instead of
`.unwrap()`, so a setup failure is distinguishable in principle from an
assertion failure further down; the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/testing.html#the-test-attribute)
documents `()` and `Result<(), E>` (with `E: Debug`) as the two accepted
return types for a `#[test]` function.

## Explanation (Embedded)

`#[test]` itself is ordinary attribute syntax, but the harness `cargo
test` builds around it is squarely a `std` feature: each discovered test
is run on its own OS thread (`std::thread`), and the harness catches
per-test unwinding panics and prints a pass/fail summary from a hosted
process — none of which exists on a bare `#![no_std]` target, which has no
thread scheduler, no unwinding-catching runtime, and typically no
attached terminal to print a report to. This is why on-target unit
testing normally isn't done by compiling the ordinary `#[test]` harness
for the microcontroller itself.

Two complementary answers cover the resulting gap, and most real embedded
crates lean on both:

**Host-testing split.** The majority of a typical embedded crate's
genuinely *testable* logic — protocol/frame parsing, checksum/CRC
computation, state-machine transitions, unit conversions — doesn't
actually touch hardware, and is written so it also compiles for the host:
kept dependent only on `core`/`alloc` rather than a specific chip's
peripherals, or placed behind a `#[cfg(test)]` module that pulls in `std`
only for the test build. That code is then tested with a completely
ordinary `cargo test`, run on the development machine's own architecture,
with the full `#[test]`/`#[ignore]`/`#[should_panic]` machinery available
exactly as on any other crate — nothing about the code eventually running
on an ARM Cortex-M or RISC-V target changes how it's tested.

**`defmt-test` (or `embedded-test`) for genuinely on-target tests.** For
code that *can't* be meaningfully tested without real hardware — a
driver's actual register writes, timing against a real peripheral, an
interrupt handler — `defmt-test` provides a `#[test]`-shaped attribute
macro of its own: functions marked with it run for real on the target
microcontroller, flashed and executed via a debug probe (`probe-rs`), with
pass/fail results and any logged output sent back to the host over RTT
(or semihosting) rather than printed by a hosted process. It deliberately
mirrors `cargo test`'s shape — a `#[tests]` module, `#[test]` functions
inside it, an optional `#[init]` fixture — so the authoring experience is
close to ordinary unit testing, but every test genuinely executes on the
chip rather than the host CPU. `embedded-test` fills the same niche with
a similar model. Both need a debug probe physically attached to real
hardware to run at all, which is a meaningfully different CI/development
story than `cargo test`'s "just run it, no hardware needed."

## Usage examples (Embedded)

### Host-testing hardware-independent logic with plain #[test]

```
// This function only manipulates bytes — no peripheral access — so it's tested on the host.
pub fn parse_frame_length(header: [u8; 2]) -> u16 {
    u16::from_be_bytes(header)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] // <- runs with an ordinary `cargo test` on the host, not on the target
    fn parses_a_two_byte_length_header() {
        assert_eq!(parse_frame_length([0x01, 0x2C]), 0x012C);
    }
}
```

### Running a test on real hardware with defmt-test

```
#![no_std]
#![no_main]

#[defmt_test::tests] // <- marks this module's functions as on-target tests, run via a debug probe
mod tests {
    use defmt::assert_eq;

    #[test] // <- executes for real on the microcontroller; pass/fail reported back over RTT
    fn adc_reads_within_expected_range() {
        let sample = read_adc_channel(0);
        assert_eq!(sample <= 4095, true, "12-bit ADC sample out of range");
    }
}
```
