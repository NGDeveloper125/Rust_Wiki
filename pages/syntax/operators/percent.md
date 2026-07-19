---
title: "%"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["%="]
see_also: ["%="]
---

## Explanation

`%` is the remainder operator, overloadable via `std::ops::Rem`:

```
let r = 7 % 2; // 1
```

It's the *remainder*, not strictly modulo — for negative operands, the
result takes the sign of the dividend (`-7 % 2 == -1`, not `1`), which
differs from the mathematical modulo used by some other languages. Like
`/`, `%` panics on division by zero for integers.

## Basic usage example

```
let r = 7 % 2; // <- `%` computes the remainder
```

**Restriction:** dividing (or taking the remainder) by zero panics
unconditionally for integers, even in release builds.

## Embedded Rust Notes

**Full support.** `Rem` lives in `core::ops` — same software-division
caveat as [`/`](slash.md) on dividerless targets.
