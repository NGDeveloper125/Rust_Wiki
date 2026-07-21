---
title: "Floating-point literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [float-suffixes, integer-decimal]
see_also: [float-suffixes]
---

## Explanation

A floating-point literal requires a decimal point (`1.0`, `0.1`), an
exponent (`123.0E+2`), or a type suffix — otherwise it's parsed as an
integer. A trailing decimal point alone is enough (`2.`).

Without a suffix, the default type is `f64`, not `f32` — the opposite
default from many other languages. A literal like `2.` (decimal point,
no digits after) is legal on its own, but a method can't be called on it
directly — `2.abs()`, `(2.).abs()`, and `2. .abs()` all fail as an
ambiguous numeric type. To call a method, use a fully written, suffixed
literal instead: `2.0_f64.abs()`.

## Usage examples

### Defaulting to f64

```
let temp = 36.6; // <- float literal: the decimal point makes this an `f64` by default
```

You can't call a method on a bare `2.` — `2.abs()`,
`2 .abs()`, and `(2.).abs()` all fail as an ambiguous numeric type. Write
a suffixed literal like `2.0_f64.abs()` when a method call is needed.

### Numeric computation

A unit-conversion formula reads clearest when every literal in it is
visibly floating-point, including whole numbers.

```
fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 1.8 + 32.0 // <- float literals: `1.8` and `32.0` are unambiguously f64, not coerced ints
}

let reading = celsius_to_fahrenheit(21.5);
```

Writing whole-number operands as `32.0` rather than
`32` keeps every value in the expression visibly floating-point, so a
type mismatch shows up as a compile error at the exact spot it was
introduced instead of relying on inference to paper over it.

### Validating input

Float equality is unreliable, so validating a measurement against a
target uses a small tolerance instead of `==`.

```
const EPSILON: f64 = 1e-6; // <- float literal: exponent form, a tiny comparison tolerance

fn is_close_to_target(measured: f64, target: f64) -> bool {
    (measured - target).abs() < EPSILON
}

assert!(is_close_to_target(98.6000001, 98.6));
```

Direct `==` on floats fails for values that are
mathematically equal but differ in their last bit due to rounding;
comparing against an epsilon threshold is the standard workaround, which
is why [Clippy's `float_cmp` lint](https://rust-lang.github.io/rust-clippy/master/index.html#float_cmp)
flags direct float equality checks in the first place.

## Embedded Rust Notes

**Full support**, but worth checking your target: many microcontrollers
have no hardware floating-point unit, so `f32`/`f64` arithmetic compiles
to (much slower) software emulation routines. Fixed-point integer
arithmetic is a common alternative on FPU-less targets.
