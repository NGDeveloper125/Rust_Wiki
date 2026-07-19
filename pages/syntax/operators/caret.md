---
title: "^"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["^="]
see_also: ["^="]
---

## Explanation

`^` is bitwise exclusive OR (XOR) between integers, overloadable via
`std::ops::BitXor`:

```
let x = 0b1010 ^ 0b0110; // 0b1100
```

Also commonly used with `bool` as an XOR/"exactly one of" operator, since
`BitXor` is implemented for `bool` as well as the integer types (unlike
`&&`/`||`, `^` never short-circuits — both operands are always evaluated).

## Basic usage example

```
let x = 0b1100 ^ 0b1010; // <- bitwise XOR: bits that differ become 1
```

## Embedded Rust Notes

**Full support.** `BitXor` lives in `core::ops` — bit-toggling a hardware
register (`reg ^= mask`) is common in embedded code, e.g. toggling an
output pin.
