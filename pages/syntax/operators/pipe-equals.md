---
title: "|="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["|"]
see_also: ["|"]
---

## Explanation

`|=` assigns the bitwise OR of the left and right operands in place,
overloadable via `std::ops::BitOrAssign`. Commonly used to set flag bits
in a bitmask.

```
let mut flags = 0b1000u8;
flags |= 0b0010; // flags is now 0b1010
```

## Basic usage example

```
let mut flags = 0b1000u8;
flags |= 0b0010; // <- `|=` ORs the right operand into `flags` in place
```

## Embedded Rust Notes

**Full support.** `BitOrAssign` lives in `core::ops` — setting flag bits
in place (`reg |= mask`) is routine embedded register manipulation.
