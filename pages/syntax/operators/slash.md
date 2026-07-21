---
title: "/"
kind: operator
embedded_support: full
groups: [Arithmetic, Basics]
related_concepts: [Operator overloading]
related_syntax: ["/="]
see_also: ["/="]
---

## Explanation

`/` is arithmetic division, overloadable via `std::ops::Div`.

Integer division truncates rather than rounding; `7 / 2 == 3` and
`-7 / 2 == -3`. Dividing an integer by zero panics unconditionally (even
in release builds) rather than producing infinity or undefined behavior;
floating-point division by zero instead follows IEEE 754 and produces
`inf`, `-inf`, or `NaN`.

## Usage examples

### Dividing two integers with truncation

```
let quotient = 7 / 2; // <- `/` divides, truncating toward zero
```

**Restriction:** dividing an integer by zero panics unconditionally, even
in release builds.

### Numeric computation

Computing an average from a sum and a count is ordinary `/`, and the
truncation behavior this page's Explanation describes is exactly why
integer division needs a moment's thought before it's used for anything
that isn't meant to be a whole number.

```
let total_ms: u32 = 1_205;
let sample_count: u32 = 4;

let average_ms = total_ms / sample_count; // <- `/` truncates: 301, not 301.25
assert_eq!(average_ms, 301);

let average_ms_f = total_ms as f64 / sample_count as f64; // 301.25, precise
```

Casting to a float before dividing is the standard fix
when truncation would silently discard a meaningful fraction — integer
`/` always rounds toward zero, never to the nearest value, a contrast the
[`div_euclid` docs](https://doc.rust-lang.org/std/primitive.i32.html#method.div_euclid)
spell out against Euclidean division.

### Handling and propagating errors

Where the divisor isn't guaranteed to be nonzero — say, an average
computed over a possibly-empty batch of readings — `checked_div` turns
the panic this page warns about into an `Option` the caller must handle.

```
fn average(total: i32, count: i32) -> Option<i32> {
    total.checked_div(count) // returns None instead of panicking when count == 0
}

assert_eq!(average(100, 4), Some(25));
assert_eq!(average(100, 0), None); // no panic: an empty batch is a normal case
```

`checked_div` makes the zero-divisor case an explicit
branch instead of an unconditional panic — the
[standard library docs](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_div)
document that it returns `None` on a zero divisor. This is worth reaching
for whenever the divisor comes from data the function doesn't fully control
(as opposed to a compile-time-known nonzero constant, where bare `/` is
fine).

## Explanation (Embedded)

`Div` lives in `core::ops`, so `/` means exactly the same thing under
`#![no_std]` — truncation toward zero, and an unconditional panic on
integer division by zero, both unchanged. The genuinely embedded-specific
angle is the hardware: many small microcontroller cores (Cortex-M0/M0+,
for instance) have no hardware integer divider at all, so a runtime `/`
lowers to a software division routine — potentially hundreds of cycles,
versus one or a few instructions on a core that does have a divider
(Cortex-M3 and up). Where the divisor is known at compile time and happens
to be a power of two, the compiler already turns `/` into a shift for you
at no extra effort; where it's "constant enough" but not a power of two —
a fixed sample rate, say — computing a reciprocal multiply-and-shift by
hand and using that instead of a runtime `/` is a real, if micro-,
optimization worth knowing about on a divider-less core. The same story
applies to floating-point `/` on a core with no hardware FPU: it lowers to
a softfloat routine, which is one more reason fixed-point/integer
arithmetic is often preferred over `f32`/`f64` in latency-sensitive
embedded code.

## Usage examples (Embedded)

### Dividing on a core without a hardware divider

```
fn average_temp(total: i32, count: i32) -> i32 {
    total / count // <- `/` here compiles to a software-division routine on a divider-less Cortex-M0
}
```

### Replacing a fixed power-of-two divisor with an explicit shift

```
const SAMPLE_RATE_SHIFT: u32 = 10; // sample rate is 1024 Hz == 2^10

fn ms_from_samples(samples: u32) -> u32 {
    (samples * 1000) >> SAMPLE_RATE_SHIFT // stands in for `samples * 1000 / 1024`; no divide instruction emitted
}
```
