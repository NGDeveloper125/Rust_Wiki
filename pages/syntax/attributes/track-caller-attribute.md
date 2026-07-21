---
title: "#[track_caller]"
kind: attribute
embedded_support: full
groups: ["Compiler Hints & Limits", "Memory & Unsafe"]
related_concepts: []
related_syntax: [fn, "panic!"]
see_also: []
---

## Explanation

`#[track_caller]` is placed on a `fn` item and changes what
`core::panic::Location::caller()` reports when called anywhere inside
that function's body (directly, or transitively through other
`#[track_caller]` functions it calls): instead of reporting the source
location where `Location::caller()` itself is written, it reports the
location of the **call site that invoked the `#[track_caller]`
function** — the caller's line, not the function's own definition.

This is precisely the mechanism behind one of the most-noticed pieces of
polish in the standard library: calling `.unwrap()` on a `None`, or
indexing a `Vec` out of bounds, reports the panic at *your* line of code
— the line in your own project where you wrote `.unwrap()` or `v[i]` —
rather than pointing uselessly at a line deep inside the standard
library's implementation of `Option::unwrap` or `Index for Vec`. Every
one of those standard library functions is itself marked
`#[track_caller]`; without it, a panic location would always point at the
one line inside `core`/`std` where the `panic!` macro (which reads
`Location::caller()` internally to build its message) is actually
written, which is true but useless to a caller trying to find the bug in
their own code.

The attribute is transitive in a specific sense: if a `#[track_caller]`
function `f` calls another `#[track_caller]` function `g`, and `g` reads
`Location::caller()`, that call reports the location that called `f` —
the location information passes through the chain of `#[track_caller]`
calls to whoever ultimately triggered the outermost one. A regular
(non-`#[track_caller]`) function in the middle of that chain breaks the
chain: it reports its own call site to whatever it calls, not the
location further up that called *it*.

This has no runtime cost in the way it sounds like it might — no stack
unwinding or backtrace capture happens just from adding the attribute.
The caller's location is passed as an implicit, extra piece of
information at the call site itself (conceptually similar to an extra
hidden argument), resolved entirely at compile time; it's a
zero-overhead, compile-time-wired mechanism, not a runtime stack walk.

`#[track_caller]` cannot be combined with certain other constructs — most
notably it has no effect through a `dyn Trait` call, since the concrete
`Location` wiring is resolved statically per call site and a trait object
call has erased which concrete call site is calling through it.

## Usage examples

### Reporting the caller's location from a panic

```
#[track_caller] // <- makes Location::caller() below report the CALLER's line, not this one
fn require_positive(value: i32) {
    if value <= 0 {
        let loc = std::panic::Location::caller();
        panic!("value must be positive, got {value} at {loc}");
    }
}

fn main() {
    require_positive(-1); // the panic message reports THIS line, not the one inside require_positive
}
```

### Validating input

A configuration-loading helper used throughout an application validates
a port number and panics on an invalid one; marking it `#[track_caller]`
makes every panic point at the call site that passed the bad value,
mirroring exactly how `Option::unwrap` reports the caller's line instead
of its own.

```
#[track_caller] // <- panics from inside this function report the CALLER's source location
fn require_valid_port(port: u32) -> u16 {
    u16::try_from(port).unwrap_or_else(|_| {
        panic!("port {port} is out of range for u16") // reports the call site below, not this line
    })
}

fn main() {
    let port = require_valid_port(70_000); // <- the panic message points here
    println!("listening on {port}");
}
```

Without `#[track_caller]`, every misconfigured call
anywhere in a large codebase would report the same unhelpful location —
the `panic!` line inside `require_valid_port` itself — instead of the
call site that actually passed the bad value; the
[std documentation for `#[track_caller]`](https://doc.rust-lang.org/std/panic/struct.Location.html#method.caller)
describes this as exactly the mechanism `Option::unwrap`/`expect` and
slice indexing use internally, and applying it to custom
validation/assertion helpers extends the same caller-friendly diagnostics
to application code.

### Testing

A custom test-assertion helper used across a test suite should report a
failed assertion at the line inside the *test* that called it, not at the
line inside the shared helper function every test calls into.

```
#[track_caller] // <- a failed assertion here reports the calling test's line, not this function's
fn assert_within_tolerance(actual: f64, expected: f64, tolerance: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tolerance,
        "expected {expected} +/- {tolerance}, got {actual}"
    );
}

#[test]
fn sensor_reading_matches_calibration() {
    let reading = 21.4;
    assert_within_tolerance(reading, 21.5, 0.05); // <- failure reports THIS line
}
```

`assert!` itself is `#[track_caller]`, so without
marking `assert_within_tolerance` the same way, a failure would report
the `assert!` line inside the shared helper — the same unhelpful,
one-size-fits-all location for every test that uses it — instead of the
specific test and assertion that actually failed; this is the standard
pattern the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/codegen.html#the-track_caller-attribute)
describes for custom assertion helpers that want to preserve
`assert!`-quality diagnostics through a layer of indirection.

## Embedded Rust Notes

**Full support.** `#[track_caller]` and `Location::caller()` are both in
`core`, with zero dependency on `alloc`/`std` or an OS, so the mechanism
works identically in `#![no_std]`. It's arguably more valuable there: a
`#![no_std]` panic handler often has no backtrace or debugger attached by
default, so the file/line a `#[track_caller]`-annotated helper reports
may be the only diagnostic information available about where a
validation failure actually originated.
