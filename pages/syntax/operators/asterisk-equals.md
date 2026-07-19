---
title: "*="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["*"]
see_also: ["*"]
---

## Explanation

`*=` multiplies the left operand by the right in place, overloadable via
`std::ops::MulAssign`.

```
let mut x = 5;
x *= 2; // x is now 10
```

Unrelated to the dereference sense of `*` — `*=` is purely the compound
arithmetic-assignment operator; there is no "deref-assign" reading of
this token.

## Embedded Rust Notes

**Full support.** `MulAssign` lives in `core::ops` — no `std` dependency.
