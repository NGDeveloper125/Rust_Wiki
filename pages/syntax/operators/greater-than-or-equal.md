---
title: ">="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["<", "<=", ">"]
see_also: [">"]
---

## Explanation

`>=` is the greater-than-or-equal comparison, part of the same
`std::ops::PartialOrd` trait as `<`, `<=`, and `>`.

```
if a >= b { ... }
```

## Basic usage example

```
let a = 5;
let b = 3;
let ok = a >= b; // <- true if `a` is greater than or equal to `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a >= b >= c` doesn't compile; write `a >= b && b >= c` instead.

## Embedded Rust Notes

**Full support.** `PartialOrd` lives in `core::cmp` — no `std` dependency.
