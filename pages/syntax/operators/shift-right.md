---
title: ">>"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: [">>=", "<<"]
see_also: ["<<"]
---

## Explanation

`>>` is the right-shift operator, overloadable via `std::ops::Shr`:

```
let x = 8u8 >> 3; // 1
```

For unsigned integers this is always a logical shift (zero-fill); for
signed integers it's an arithmetic shift (sign-bit-fill), matching the
type's own notion of sign. See [`<<`](shift-left.md) for the shared notes
on out-of-range shift amounts.

## Embedded Rust Notes

**Full support.** `Shr` lives in `core::ops` — no `std` dependency.
