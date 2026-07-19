---
title: "-="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["-"]
see_also: ["-"]
---

## Explanation

`-=` subtracts the right operand from the left in place, overloadable via
`std::ops::SubAssign`.

```
let mut x = 5;
x -= 2; // x is now 3
```

See [`+=`](plus-equals.md) for the general notes on compound assignment
operators (mutable place required, potentially distinct impl from the
non-assigning operator).

## Embedded Rust Notes

**Full support.** `SubAssign` lives in `core::ops` — no `std` dependency.
