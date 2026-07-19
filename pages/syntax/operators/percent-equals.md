---
title: "%="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["%"]
see_also: ["%"]
---

## Explanation

`%=` assigns the remainder of the left operand divided by the right,
overloadable via `std::ops::RemAssign`.

```
let mut x = 7;
x %= 2; // x is now 1
```

## Basic usage example

```
let mut x = 7;
x %= 2; // <- `%=` assigns the remainder of `x / 2` back into `x`
```

## Embedded Rust Notes

**Full support.** `RemAssign` lives in `core::ops` — no `std` dependency.
