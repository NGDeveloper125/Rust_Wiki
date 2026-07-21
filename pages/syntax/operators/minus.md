---
title: "-"
kind: operator
embedded_support: full
groups: [Arithmetic, Basics]
related_concepts: [Operator overloading]
related_syntax: ["-="]
see_also: ["-="]
---

## Explanation

`-` is both binary subtraction and unary negation, disambiguated by
whether a left operand is present.

Binary `-` is overloadable via `std::ops::Sub`; unary `-` via
`std::ops::Neg`. They're separate traits — a type can implement one
without the other. Negation only applies to signed numeric types by
default; unsigned integers (`u32`, etc.) have no `Neg` impl, so `-x` where
`x: u32` is a compile error, not a runtime panic. Integer overflow from
subtracting runtime values (e.g. `a - b` where `b > a` for unsigned
types) panics in debug builds and wraps in release builds, unless
explicitly guarded with methods like `checked_sub`. A constant expression
that overflows, such as the literal `0u8 - 1`, never gets that far — it's
rejected at compile time by the deny-by-default `arithmetic_overflow`
lint.

## Usage examples

### Subtracting two numbers

```
let diff = 10 - 3; // <- binary `-` subtracts the right operand
```

**Restriction:** subtracting runtime values past a type's minimum panics
in debug builds; release builds wrap instead, unless you use a checked
method like `checked_sub`. A constant expression like `0u8 - 1` is a
compile error instead (deny-by-default `arithmetic_overflow` lint).

### Numeric computation

A sensor reading's delta from a setpoint is a common place binary `-`
shows up — and a good spot to reach for `checked_sub` instead of the bare
operator once the operands might not stay in a safe order.

```
fn temperature_delta(setpoint: i32, reading: i32) -> i32 {
    reading - setpoint // <- binary `-`: how far off target, signed
}

fn safe_battery_drop(before: u8, after: u8) -> Option<u8> {
    before.checked_sub(after) // avoids the panic/wrap that bare `-` risks on u8
}

let delta = temperature_delta(20, 17); // -3: three degrees below setpoint
```

Signed deltas (`i32`, not `u32`) let "below setpoint"
be a negative number instead of an underflow; for unsigned subtraction
where the operands' order isn't guaranteed, `checked_sub` turns a
potential panic into an `Option` the caller must handle, per the
[standard library docs](https://doc.rust-lang.org/std/primitive.u8.html#method.checked_sub).

### Working with collections

Computing pairwise differences between consecutive elements — e.g. the
gaps between successive sensor readings — is a natural fit for
`windows(2)` paired with binary `-`.

```
let readings = [10.0, 12.5, 11.0, 13.5];

let deltas: Vec<f64> = readings
    .windows(2)
    .map(|pair| pair[1] - pair[0]) // <- `-` computes each consecutive gap
    .collect();

assert_eq!(deltas, vec![2.5, -1.5, 2.5]);
```

`windows(2)` avoids manually indexing with `i` and
`i - 1` (an off-by-one hazard), letting `-` do the actual subtraction
while the iterator adapter handles bounds — see the
[standard library docs](https://doc.rust-lang.org/std/primitive.slice.html#method.windows)
for the general `windows` pattern over any fixed-size grouping.

## Explanation (Embedded)

`Sub`/`Neg` live in `core::ops`, so both binary subtraction and unary
negation work identically under `#![no_std]`. The overflow behavior this
page's classic Explanation describes is the detail that matters more in
firmware than in most hosted code: a release build — the profile actually
flashed to a device — has overflow checks off by default, so `a - b`
wraps silently instead of panicking. In a desktop program a debug build
usually catches an underflow during development; a device already in the
field can't be recompiled in debug to catch the same bug after shipping.
That makes `checked_sub`/`wrapping_sub`/`saturating_sub` a more
load-bearing idiom in embedded code than the occasional defensive check it
is elsewhere — particularly for anything computed from a free-running
hardware counter or an external sensor reading, where the operands aren't
fully under the program's control.

## Usage examples (Embedded)

### Computing elapsed ticks from a free-running hardware counter

A hardware tick counter register wraps back to `0` after its maximum
value, so plain `-` would occasionally underflow across that wrap;
`wrapping_sub` treats the wraparound as intentional and yields the
correct short elapsed time regardless.

```
fn ticks_since(now: u32, last: u32) -> u32 {
    now.wrapping_sub(last) // <- wrapping `-`: still correct even after `now` has wrapped past u32::MAX
}
```

### Guarding a sensor delta against a silent release-mode wrap

```
fn safe_delta(current: u16, baseline: u16) -> Option<u16> {
    current.checked_sub(baseline) // <- None instead of a silently wrapped value on a shipped device
}
```
