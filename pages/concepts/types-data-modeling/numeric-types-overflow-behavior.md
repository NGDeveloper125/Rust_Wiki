---
title: "Numeric types & overflow behavior"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Numeric Safety"]
related_syntax: [integer-suffixes, float-suffixes]
see_also: []
---

## Explanation

Rust's integer types are explicit about width and signedness —
`u8`/`i8` through `u128`/`i128`, plus pointer-sized `usize`/`isize` — with
no single generic "number" type and no implicit widening between
different integer types (adding a `u8` and a `u32` directly is a compile
error; an explicit `as` cast or `.into()` conversion is required).

Overflow behavior is deliberately different between build profiles: in a
debug build, an operation that overflows its type's range (`255u8 + 1`)
panics immediately, surfacing the bug during development. In a release
build, the same operation instead **wraps** silently (`255u8 + 1 == 0`)
for performance reasons — checking every arithmetic operation at runtime
in optimized code would cost real, measurable overhead. Where either
behavior isn't good enough — you need guaranteed, defined behavior
regardless of build profile — explicit methods make the choice visible in
the code itself: `checked_add` (returns `None` on overflow),
`wrapping_add` (always wraps), `saturating_add` (clamps to the type's
max/min), and `overflowing_add` (returns the wrapped value plus a bool
flag).

This design trades a small amount of implicit safety (debug-only
overflow panics) for explicitness everywhere it actually matters, rather
than picking one runtime behavior and making every caller pay for it
unconditionally.

## Basic usage example

```
let a: u8 = 250;
let b: u8 = 10;

let sum = a.checked_add(b); // <- None: 260 doesn't fit in a u8, caught explicitly
println!("{sum:?}");
```

**Restriction:** writing plain `a + b` here instead hides the same
problem — it panics in a debug build but silently wraps to `4` in a
release build, so the two profiles behave differently unless you use an
explicit `checked_`/`wrapping_`/`saturating_` method.

## Embedded Rust Notes

**Full support.** All defined in `core`, no `std` dependency. Overflow
behavior is worth extra attention in embedded code: register widths
often dictate the natural integer type (`u16` for a 16-bit ADC reading,
for example), and the debug-panics/release-wraps split still applies —
safety-critical embedded code frequently uses explicit
`checked_`/`saturating_` arithmetic deliberately, rather than relying on
build-profile-dependent default behavior.
