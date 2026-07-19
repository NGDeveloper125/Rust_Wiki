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

## Embedded Rust Notes

**Full support.** `PartialOrd` lives in `core::cmp` — no `std` dependency.
