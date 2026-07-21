---
title: "Panic & unwinding"
area: "Error Handling"
embedded_support: partial
groups: ["Error Handling", "Handling Errors & Failure"]
related_syntax: ["panic!"]
see_also: ["Result<T, E>", "The Error trait", "RAII & the Drop trait"]
---

## Explanation

`panic!` is Rust's mechanism for unrecoverable errors — bugs, broken
invariants, states the program has no sensible way to continue from.
Unwinding is the default runtime behavior when a panic fires: the stack
unwinds frame by frame, running `Drop` for every live value on the way
out (see [RAII & the `Drop` trait](../ownership-borrowing/raii-and-drop.md)),
until it either reaches a `catch_unwind` boundary or exits the thread —
for the main thread, that means the process exits with a nonzero status.

It exists to draw a hard line between two very different kinds of
failure. A [`Result`](result.md) models something a caller can
reasonably plan for and react to — a file that might not exist, a port
that might be busy. A panic models a condition that should never happen
if the program is correct: an index past the end of a slice, an
assertion the code itself claims can never fail. Blurring the two — for
example, returning a sentinel value instead of panicking on a broken
invariant — hides bugs instead of surfacing them.

The mental model to keep is that panicking is not Rust's exception
system for everyday control flow. `catch_unwind` exists, but it's meant
for isolating boundaries — a thread-pool worker that shouldn't take down
the whole pool, an FFI boundary that must not let a panic cross into
foreign code — not for routinely "catching" failures the way `try`/`catch`
is used elsewhere. If a failure is something a caller should reasonably
plan for, the right tool is `Result`, not a panic a caller is expected to
catch.

Because panicking and returning `Result` express such different
contracts, choosing between them is a real API design decision, not a
style preference: a function should panic only on a documented
precondition the caller is responsible for satisfying, and return
`Result` for everything a caller might legitimately trigger just by
passing in ordinary, untrusted data.

Rust also supports two different panic *strategies*, chosen per build:
`panic = "unwind"` (the default on hosted targets, described above) and
`panic = "abort"`, which terminates the process immediately without
running destructors or unwinding the stack at all — smaller and faster,
at the cost of losing `catch_unwind` and guaranteed cleanup. Which one
applies matters most once a target has no OS to catch an unwound panic
at the top of a thread — see Embedded Rust Notes below.

## Basic usage example

```
fn nth_element(values: &[i32], index: usize) -> i32 {
    values[index] // <- panics with an unrecoverable error if `index` is out of bounds
}

let scores = [10, 20, 30];
let first = nth_element(&scores, 0);
```

## Best practices & deeper information

### Scenario: Testing

A `should_panic` test documents and locks in that calling a function
with an out-of-range index is expected to panic, not silently return a
wrong value.

```
fn nth_reading(readings: &[f64], index: usize) -> f64 {
    readings[index] // <- indexing panics on out-of-bounds access
}

#[test]
#[should_panic(expected = "index out of bounds")] // <- asserts the panic happens, with a matching message
fn rejects_out_of_range_index() {
    let readings = [21.5, 22.0];
    nth_reading(&readings, 5);
}
```

**Why this way:** `should_panic` with `expected` pins down both that a
panic occurs and roughly what it says, so a future change that turns the
panic into a silent wrong answer (or an unrelated panic) fails the test —
the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-for-panics-with-should_panic)
documents `should_panic` for exactly this.

### Scenario: Handling and propagating errors

A lookup that can reasonably fail (an index a caller didn't check first)
gets a checked alternative that returns `Option`, while the panicking
form is reserved for callers who documented that they already know the
index is valid — and the difference is written into the docs, not left
implicit.

```
struct Buffer {
    samples: Vec<f64>,
}

impl Buffer {
    /// Panics if `index >= self.samples.len()`; the caller is expected
    /// to have checked `len()` first — this is a contract violation, not
    /// a recoverable condition.
    fn sample(&self, index: usize) -> f64 {
        self.samples[index] // <- panics: violating a documented precondition is a bug, not a Result-worthy failure
    }

    /// Never panics: an out-of-range index is a normal, expected outcome
    /// for a caller that doesn't know `len()` in advance.
    fn get_sample(&self, index: usize) -> Option<f64> {
        self.samples.get(index).copied()
    }
}
```

**Why this way:** the API Guidelines'
[C-FAILURE item](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure)
calls for documenting exactly when a function panics, so callers can
tell a documented precondition violation (fine to panic on) apart from an
ordinary failure mode (which should return `Result` or `Option` instead)
— offering both a panicking and a checked variant, as `Vec` itself does
with indexing versus `get`, gives callers the choice.

### Scenario: Validating input

An internal invariant — a percentage field must stay within `0..=100` —
is enforced with `debug_assert!` so violations are caught hard in
development builds, without paying the cost in release builds.

```
struct Discount {
    percent_off: u8,
}

impl Discount {
    fn new(percent_off: u8) -> Self {
        debug_assert!(percent_off <= 100, "percent_off must be 0..=100, got {percent_off}");
        // <- panics in debug builds on a broken invariant; compiled out of release builds
        Discount { percent_off }
    }
}

let discount = Discount::new(15);
```

**Why this way:** `debug_assert!` is for catching programmer errors
during development at effectively zero cost in release builds, distinct
from validating genuinely untrusted external input, which should still
return `Result` even in release — a distinction the
[std docs draw explicitly](https://doc.rust-lang.org/std/macro.debug_assert.html)
between `assert!` and `debug_assert!`.

## Explanation (Embedded)

**Partial support** — `panic!` itself fires identically, but unwinding, the
default *strategy* described in the classic Explanation, is generally not
available at all. Bare metal has no OS process for an unwind runtime to
unwind into, and most bare-metal targets lack the unwind-table support a
`panic = "unwind"` build even needs — so `panic = "abort"` is close to a
universal default in embedded profiles, not merely one of two equally
viable choices the way it can be on a hosted target. The practical
consequence of that design decision: `Drop` does not run on the way out of
a panic (no flushing a buffer, no releasing a peripheral lock through a
guard's destructor), and `catch_unwind` isn't available as an escape hatch
at all — a panic anywhere in the firmware takes the whole device down, not
just a thread.

Every `#![no_std]` binary must supply exactly one `#[panic_handler]`
function, and that function decides what "taking the device down" means
in practice — there's no shell process boundary to hand an exit code to.
Typical implementations park the core in an infinite loop, report the
message over a debug channel first, or trigger a watchdog/software reset so
the device comes back up in a known state rather than staying hung. See
[`panic!`](../../syntax/macros/panic-macro.md) for the attribute mechanics
and the concrete handler crates (`panic-halt`, `panic-itm`, `panic-probe`)
— this page is about the underlying *why*: on a bare-metal target a panic
isn't a recoverable event worth designing a catch-all handler around, it's
closer to the hardware equivalent of a fatal fault, so the classic page's
principle — reserve panics for broken invariants, not everyday failure —
applies with even less of a safety net beneath it.

## Basic usage example (Embedded)

```
#![no_std]
#![no_main]

use panic_halt as _; // <- the #[panic_handler] this build runs; see panic! (syntax page) for handler choices
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let valve_percent: u8 = 150;
    if valve_percent > 100 {
        panic!("valve position {valve_percent}% exceeds 100%"); // <- broken precondition: no OS underneath to catch an unwind
    }
    loop {}
}
```

## Best practices & deeper information (Embedded)

### Scenario: Handling and propagating errors

A sample buffer offers both a panicking direct-index method, for a caller
that has already checked bounds against a known frame size, and a checked
`Option`-returning one for everything else — the same split as the
classic page's `Buffer`, but with a higher cost attached to guessing wrong
about which one applies.

```
struct SampleBuffer {
    samples: [i16; 32],
    len: usize,
}

impl SampleBuffer {
    /// Panics if `index >= self.len`; only call this after checking `len()` —
    /// on this target `panic = "abort"`, so this halts/resets the device.
    fn sample(&self, index: usize) -> i16 {
        assert!(index < self.len, "index {index} out of bounds for len {}", self.len);
        self.samples[index] // <- indexing panics on out-of-bounds, same as the classic page's Buffer example
    }

    /// Never panics: returns None for an out-of-range index instead.
    fn get_sample(&self, index: usize) -> Option<i16> {
        if index < self.len {
            Some(self.samples[index])
        } else {
            None
        }
    }
}
```

**Why this way:** offering both forms keeps the abort-on-panic cost — a
full device halt or reset, not a `catch_unwind`-able exception — reserved
for genuine precondition violations, while an ordinary "might be
out-of-range" situation uses the checked `Option` form instead, the same
[C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure)-driven
split as the classic page's example.

### Scenario: Validating input

An internal PWM duty-cycle invariant is checked with `debug_assert!`
rather than `assert!`, the same idiom as the classic page — but on a
target with kilobytes of flash rather than gigabytes of disk, the cost of
compiling the check and its formatted message into every release binary is
a concrete, not merely theoretical, reason to reach for it.

```
struct PwmChannel {
    duty_percent: u8,
}

impl PwmChannel {
    fn new(duty_percent: u8) -> Self {
        debug_assert!(duty_percent <= 100, "duty_percent must be 0..=100, got {duty_percent}");
        // <- compiled out of release builds: no message-formatting code bloats the release binary's flash image
        PwmChannel { duty_percent }
    }
}
```

**Why this way:** `debug_assert!` keeps the check and its formatted message
entirely out of the release binary, which matters more on a target
measuring flash in kilobytes than on a hosted binary where a few extra
bytes of panic-message string are irrelevant — the same
[std docs' distinction](https://doc.rust-lang.org/std/macro.debug_assert.html)
the classic page cites, with the code-size cost made concrete.
