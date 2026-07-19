---
title: "+="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["+"]
see_also: ["+"]
---

## Explanation

`+=` adds the right operand to the left in place, overloadable via
`std::ops::AddAssign`:

```
let mut x = 5;
x += 1; // x is now 6
```

`x += 1` is not always exactly sugar for `x = x + 1` — types can implement
`AddAssign` differently from `Add` (e.g. to mutate in place without an
extra allocation, which matters for types like `String` or `Vec`), though
for simple numeric types the two behave identically. The left operand
must be a mutable place — `x` must be declared `let mut x`.

## Embedded Rust Notes

**Full support.** `AddAssign` lives in `core::ops` — no `std` dependency.
