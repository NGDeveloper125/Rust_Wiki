---
title: "Binary integer literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["Numeric types & overflow behavior"]
related_syntax: [integer-suffixes, digit-separator]
see_also: [integer-decimal, integer-hexadecimal, integer-octal]
---

## Explanation

A base-2 integer literal, prefixed with `0b`:

```
let flags = 0b1010_0101u8;
```

Underscores are especially common here to group digits into readable
nibbles/bytes, as shown above — purely cosmetic, no effect on the value.

## Embedded Rust Notes

**Full support.** Binary literals are extremely common in embedded code
for expressing register bit patterns and masks directly
(`0b0000_0001`) — no `std` dependency.
