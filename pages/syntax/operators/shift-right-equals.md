---
title: ">>="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: [">>"]
see_also: [">>"]
---

## Explanation

`>>=` right-shifts the left operand by the right operand's amount, in
place, overloadable via `std::ops::ShrAssign`.

```
let mut x = 8u8;
x >>= 3; // x is now 1
```

## Embedded Rust Notes

**Full support.** `ShrAssign` lives in `core::ops` — no `std` dependency.
