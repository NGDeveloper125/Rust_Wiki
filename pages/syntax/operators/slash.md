---
title: "/"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["/="]
see_also: ["/="]
---

## Explanation

`/` is arithmetic division, overloadable via `std::ops::Div`:

```
let quotient = 7 / 2; // 3 for integers — truncates toward zero
```

Integer division truncates rather than rounding; `7 / 2 == 3` and
`-7 / 2 == -3`. Dividing an integer by zero panics unconditionally (even
in release builds) rather than producing infinity or undefined behavior;
floating-point division by zero instead follows IEEE 754 and produces
`inf`, `-inf`, or `NaN`.

## Embedded Rust Notes

**Full support.** `Div` lives in `core::ops`. Worth knowing: many small
microcontrollers have no hardware integer divider, so `/` on those
targets compiles to a (slower) software division routine — profile before
assuming it's free.
