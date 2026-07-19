---
title: "<<="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["<<"]
see_also: ["<<"]
---

## Explanation

`<<=` left-shifts the left operand by the right operand's amount, in
place, overloadable via `std::ops::ShlAssign`.

```
let mut x = 1u8;
x <<= 3; // x is now 8
```

## Basic usage example

```
let mut x = 1u8;
x <<= 3; // <- `<<=` left-shifts `x` in place
```

## Embedded Rust Notes

**Full support.** `ShlAssign` lives in `core::ops` — no `std` dependency.
