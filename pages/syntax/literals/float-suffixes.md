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

`f32` and `f64` pin a floating-point literal's type explicitly, written
directly after the digits, as in `1.0f32` or `2.5_f64`.

`f64` is the default when no suffix and no other context pins the type —
it has more precision than `f32` at the cost of double the memory, and is
what Rust favors unless you specifically need `f32` (e.g. for
GPU/graphics interop or memory-constrained contexts).

## Usage examples

### Pinning a literal to f32

```
let ratio = 0.5f32; // <- `f32` suffix pins the literal's type explicitly
```

### Numeric computation

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

Without suffixes, `lerp(0.0, 10.0, 0.25)` would silently
pick `f64` for `T`, which matters when the result is headed for
`f32`-only territory like a GPU buffer — see
[float literal](float-literal.md) for the `f64`-by-default rule this
overrides.

## Explanation (Embedded)

The `f32`/`f64` suffix pins a literal's type identically under
`#![no_std]` — a lexical, compile-time rule with no dependency on `std`.
It matters more in embedded code than the choice usually does on a
desktop, though, because of the FPU asymmetry described on
[Floating-point literal](float-literal.md): most Cortex-M parts that have
a hardware FPU at all only accelerate `f32`, not `f64`. Since `f64` is
the default whenever nothing else pins the type, an unsuffixed literal
in generic numeric code can silently end up on the slow, software-emulated
`f64` path even on hardware that could have handled `f32` natively —
explicitly suffixing the literal is what actually secures the hardware
acceleration, rather than leaving it to inference.

## Usage examples (Embedded)

### Pinning ADC scaling math to f32 for hardware FPU acceleration

```
fn to_millivolts(raw: u16, vref: f32) -> f32 {
    (raw as f32 / 4095.0f32) * vref // <- `f32` suffix keeps this on the Cortex-M4F's hardware FPU path, not software f64
}
```

### Pinning a generic filter's type parameter to f32

```
fn low_pass<T>(prev: T, sample: T, alpha: T) -> T
where
    T: core::ops::Add<Output = T> + core::ops::Sub<Output = T> + core::ops::Mul<Output = T> + Copy,
{
    prev + (sample - prev) * alpha
}

let filtered = low_pass(0.0f32, 3.3f32, 0.1f32); // <- `f32` suffixes: keep T = f32 instead of defaulting the generic to f64
```
