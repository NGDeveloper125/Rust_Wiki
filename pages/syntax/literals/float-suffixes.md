---
title: "Float suffixes"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [float-literal, integer-suffixes]
see_also: [float-literal]
---

## Explanation

`f32` and `f64` pin a floating-point literal's type explicitly:

```
let a = 1.0f32;
let b = 2.5_f64;
```

`f64` is the default when no suffix and no other context pins the type —
it has more precision than `f32` at the cost of double the memory, and is
what Rust favors unless you specifically need `f32` (e.g. for
GPU/graphics interop or memory-constrained contexts).

## Basic usage example

```
let ratio = 0.5f32; // <- `f32` suffix pins the literal's type explicitly
```

## Best practices & deeper information

### Scenario: Numeric computation

In a generic numeric function, the compiler has nothing but the literals
themselves to pin the type parameter — without a suffix, every argument
would default to `f64`.

```
fn lerp<T>(a: T, b: T, t: T) -> T
where
    T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<Output = T> + Copy,
{
    a + (b - a) * t
}

let position = lerp(0.0f32, 10.0f32, 0.25f32); // <- `f32` suffix: pins T to f32, not the f64 default
```

**Why this way:** without suffixes, `lerp(0.0, 10.0, 0.25)` would silently
pick `f64` for `T`, which matters when the result is headed for
`f32`-only territory like a GPU buffer — see
[float literal](float-literal.md) for the `f64`-by-default rule this
overrides.

## Embedded Rust Notes

**Full support.** No `std` dependency — see
[Floating-point literal](float-literal.md) for the hardware-FPU caveat
that applies regardless of which suffix you pick.
