---
title: "assert! / assert_eq! / assert_ne!"
kind: macro
embedded_support: full
groups: ["Errors & Assertions", "Macros & Metaprogramming"]
related_concepts: ["Unit tests"]
related_syntax: ["==", "panic!"]
see_also: ["==", "panic!"]
---

## Explanation

`assert!(condition)` panics (via [`panic!`](panic-macro.md)) if
`condition` evaluates to `false`, and does nothing otherwise; an optional
second argument, `assert!(condition, "message {value}")`, supplies a
format-string message using the same grammar as `panic!`/`format!`,
shown instead of the default "assertion failed: `condition`" text. That
default text comes from `stringify!` (see
[`concat!` / `stringify!` / `line!` / `column!` / `file!` /
`module_path!`](introspection-macros.md)) turning the condition
expression's own tokens back into a string — which is also exactly why a
bare `assert!(a == b)` only ever reports that the *expression* `a == b`
was false, never what `a` and `b` actually *were*.

`assert_eq!(a, b)` and `assert_ne!(a, b)` close that gap for the specific,
extremely common case of comparing two values: both require `a` and `b`
to implement `PartialEq` (see [`==`](../operators/equal-equal.md)) and
`Debug`, and on failure their panic message prints both operands, labeled
`left` and `right`, using their `Debug` formatting — so a failing
`assert_eq!(total, 42)` reports the actual value of `total` alongside the
`42` it was compared against, not just the fact that they differed. This
is the concrete reason `assert_eq!`/`assert_ne!` are the default choice
for equality checks in tests instead of `assert!(a == b)`/`assert!(a !=
b)`.

## Usage examples

### Custom failure messages vs. automatic operand reporting

```
let cart_total = 24;
assert!(cart_total > 0, "cart total must be positive, got {cart_total}"); // <- custom message shown on failure
assert_eq!(cart_total, 24); // <- prints both `left` (cart_total) and `right` (24) if this ever fails
```

### Testing

A unit test for a temperature-conversion function compares the computed
and expected values with `assert_eq!`, and a follow-up check confirms
retrying a flaky request produces a different request ID with
`assert_ne!`.

```
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

fn next_request_id(previous: u64) -> u64 {
    previous + 1
}

#[test]
fn converts_boiling_point() {
    assert_eq!(celsius_to_fahrenheit(100.0), 212.0); // <- on failure, prints both the computed and expected value
}

#[test]
fn retried_request_gets_a_new_id() {
    let first = next_request_id(0);
    let retried = next_request_id(first);
    assert_ne!(first, retried); // <- on failure, prints both ids, proving they collided
}
```

The
[Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
recommends `assert_eq!`/`assert_ne!` over a bare `assert!(a == b)`
because the failure output includes both operands' `Debug`
representations, saving a debugging round trip spent re-running the test
with an added print statement.

### Validating input

A public constructor enforces a range invariant with `assert!` rather
than `debug_assert!`, because the invariant must hold in every build a
caller might ship, not only development builds.

```
pub struct SamplingRate {
    hz: u32,
}

impl SamplingRate {
    pub fn new(hz: u32) -> Self {
        // AVOID: debug_assert! is compiled out of release builds — a caller
        // shipping a release build would silently accept hz == 0.
        // debug_assert!(hz > 0, "sampling rate must be nonzero");

        assert!(hz > 0, "sampling rate must be nonzero, got {hz}"); // PREFER: always runs, even in release
        SamplingRate { hz }
    }
}

let rate = SamplingRate::new(44_100);
```

The
[std docs](https://doc.rust-lang.org/std/macro.debug_assert.html) draw
this exact line between the two macros — `assert!` is for invariants that
must never be violated in any build a user runs, reserving
`debug_assert!` for checks whose cost is only acceptable to pay during
development.

## Explanation (Embedded)

All three are `core::assert!`/`core::assert_eq!`/`core::assert_ne!` and
behave identically under `#![no_std]` — they're implemented directly on
top of `panic!`, with no dependency on `std` at all. What changes is what
happens when one fires: instead of unwinding into a hosted OS's runtime,
a failed assertion calls into whatever `#[panic_handler]` the binary
links — commonly `panic-halt` (spins the core forever) or `panic-probe`
(prints the panic message over RTT to an attached debugger before
halting or resetting). Either way there's no stack to unwind and no
process for an OS to reap; the handler decides exactly what "give up"
means for that hardware.

`debug_assert!`/`debug_assert_eq!`/`debug_assert_ne!` disappearing from
release builds is a sharper trade-off on a microcontroller than on a
desktop: release is what actually gets flashed to production hardware,
so an invariant checked only via `debug_assert!` has no second chance to
catch a violation once shipped — the same gap the classic explanation
describes, just with less margin for error on the far side of it.

## Usage examples (Embedded)

### Wiring assert! to a panic handler

```
#![no_std]
#![no_main]

use panic_halt as _; // <- #[panic_handler]: a failed assert! below spins the core forever here
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let raw_reading = read_pressure_register();
    assert!(raw_reading < 4096, "12-bit ADC reading out of range: {raw_reading}"); // <- panics into panic_halt's handler, not an unwind
    loop {}
}

fn read_pressure_register() -> u16 {
    0 // placeholder for a real register read
}
```

### Validating a peripheral precondition that must hold in every build

```
pub struct Uart {
    baud: u32,
}

impl Uart {
    pub fn new(baud: u32) -> Self {
        // AVOID: debug_assert! is compiled out of release builds — a release
        // firmware image would silently accept baud == 0 and never transmit.
        // debug_assert!(baud > 0, "baud rate must be nonzero");

        assert!(baud > 0, "baud rate must be nonzero, got {baud}"); // PREFER: checked in every build, including the one that gets flashed
        Uart { baud }
    }
}
```
