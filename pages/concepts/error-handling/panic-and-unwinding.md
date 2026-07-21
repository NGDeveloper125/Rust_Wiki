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

## Embedded Rust Notes

**Partial support.** `panic!` itself is `core::panic!` and available with
no `alloc`/`std` at all, but *unwinding* is what typically becomes
unavailable: `#![no_std]` targets almost always build with
`panic = "abort"` in their profile, since stack unwinding needs
target/OS support for unwind tables that bare-metal targets don't have.
That means `Drop` doesn't run on panic there and `catch_unwind` doesn't
exist. Every `#![no_std]` binary must also supply exactly one
`#[panic_handler]` function — replacing std's default
"print a backtrace and abort" behavior — typically resetting the device,
signaling over a diagnostic pin, or halting in a loop.
