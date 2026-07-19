---
title: "&="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["&"]
see_also: ["&"]
---

## Explanation

`&=` assigns the bitwise AND of the left and right operands in place,
overloadable via `std::ops::BitAndAssign`. Only the integer/bitwise sense
of `&` has a compound-assignment form — there's no analogous compound
form for the borrow sense of `&` (borrowing isn't a "value" that can be
compounded like this).

```
let mut flags = 0b1111u8;
flags &= 0b1010; // flags is now 0b1010
```

## Basic usage example

```
let mut flags = 0b1100u8;
flags &= 0b1010; // <- clears bits not set in the mask, in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `&=` assigns in place.

## Embedded Rust Notes

**Full support.** `BitAndAssign` lives in `core::ops` — clearing specific
bits in a register (`reg &= !mask`) is routine embedded code.
