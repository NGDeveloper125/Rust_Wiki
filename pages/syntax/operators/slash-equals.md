---
title: "/="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["/"]
see_also: ["/"]
---

## Explanation

`/=` divides the left operand by the right in place, overloadable via
`std::ops::DivAssign`.

```
let mut x = 7;
x /= 2; // x is now 3 (integer truncation, same as `/`)
```

## Basic usage example

```
let mut x = 7;
x /= 2; // <- `/=` divides `x` in place, truncating toward zero
```

## Embedded Rust Notes

**Full support.** `DivAssign` lives in `core::ops` — same
software-division caveat as [`/`](slash.md) on dividerless targets.
