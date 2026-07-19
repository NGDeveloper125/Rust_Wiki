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

## Embedded Rust Notes

**Full support.** No `std` dependency — see
[Floating-point literal](float-literal.md) for the hardware-FPU caveat
that applies regardless of which suffix you pick.
