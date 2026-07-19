---
title: "^="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["^"]
see_also: ["^"]
---

## Explanation

`^=` assigns the bitwise XOR of the left and right operands in place,
overloadable via `std::ops::BitXorAssign`. A classic use is toggling bits:
`flags ^= mask` flips exactly the bits set in `mask`.

```
let mut flags = 0b1010u8;
flags ^= 0b0110; // flags is now 0b1100
```

## Basic usage example

```
let mut flags = 0b1010u8;
flags ^= 0b0011; // <- toggles the bits set in the mask, in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `^=` assigns in place.

## Embedded Rust Notes

**Full support.** `BitXorAssign` lives in `core::ops` — no `std`
dependency.
