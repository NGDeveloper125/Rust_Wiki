---
title: "Digit separator (_)"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [integer-decimal, integer-hexadecimal, integer-octal, integer-binary]
see_also: [integer-decimal]
---

## Explanation

An underscore may be placed anywhere between digits in a numeric literal
purely to improve readability — it carries no meaning and does not affect
the value:

```
let million = 1_000_000;
let mask = 0b1010_0101_u8;
```

It may also appear directly between the digits and a type suffix
(`1_000_i64`). The only place it's disallowed is inside a tuple index
(`t.0`, not `t.0_0`) and before the first digit of the number itself —
`_1` is parsed as an identifier, not a numeric literal starting with an
underscore.

## Embedded Rust Notes

**Full support.** Purely cosmetic at compile time — no `std` dependency,
no runtime effect whatsoever.
