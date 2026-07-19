---
title: "<="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["<", ">", ">="]
see_also: ["<"]
---

## Explanation

`<=` is the less-than-or-equal comparison, provided by `std::ops::PartialOrd`
alongside `<`, `>`, and `>=` — implementing `PartialOrd` (usually via
`#[derive(PartialOrd)]`, which requires `PartialEq` as well) gives you all
four ordering operators together, not just one.

```
if a <= b { ... }
```

## Basic usage example

```
let a = 3;
let b = 5;
let ok = a <= b; // <- true if `a` is less than or equal to `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a <= b <= c` doesn't compile; write `a <= b && b <= c` instead.

## Embedded Rust Notes

**Full support.** `PartialOrd` lives in `core::cmp` — no `std` dependency.
