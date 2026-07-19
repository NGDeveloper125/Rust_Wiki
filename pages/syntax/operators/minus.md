---
title: "-"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["-="]
see_also: ["-="]
---

## Explanation

`-` is both binary subtraction and unary negation, disambiguated by
whether a left operand is present:

```
let diff = 5 - 2;   // binary: subtraction
let neg = -x;        // unary: negation
```

Binary `-` is overloadable via `std::ops::Sub`; unary `-` via
`std::ops::Neg`. They're separate traits — a type can implement one
without the other. Negation only applies to signed numeric types by
default; unsigned integers (`u32`, etc.) have no `Neg` impl, so `-x` where
`x: u32` is a compile error, not a runtime panic. In debug builds, integer
overflow from subtraction (e.g. `0u8 - 1`) panics; in release builds it
wraps, unless explicitly guarded with methods like `checked_sub`.

## Basic usage example

```
let diff = 10 - 3; // <- binary `-` subtracts the right operand
```

**Restriction:** subtracting past a type's minimum value (e.g. `0u8 - 1`)
panics in debug builds; release builds wrap instead, unless you use a
checked method like `checked_sub`.

## Embedded Rust Notes

**Full support.** `Sub`/`Neg` live in `core::ops` — no `std` dependency.
