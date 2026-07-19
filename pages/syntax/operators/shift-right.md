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

## Basic usage example

```
let x = 8u8 >> 3; // <- `>>` shifts the bits of `8u8` right by 3
```

**Restriction:** as with `<<`, shifting by an amount greater than or
equal to the type's bit width panics in debug builds.

## Embedded Rust Notes

**Full support.** `Shr` lives in `core::ops` — no `std` dependency.
