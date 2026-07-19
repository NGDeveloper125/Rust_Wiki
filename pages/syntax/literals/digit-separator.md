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

## Basic usage example

```
let value = 10_000;
//            ^ digit separator: purely cosmetic, does not affect the value
```

**Restriction:** an underscore can't be the very first character of the
literal — `_1_000` is parsed as an identifier, not a numeric literal.

## Best practices & deeper information

### Scenario: Numeric computation

A large constant like a byte limit or a Unix timestamp is much easier to
proofread once it's grouped into readable chunks.

```
const MAX_UPLOAD_BYTES: u64 = 10_485_760;   // <- digit separators: 10 MiB, grouped in thousands
const EPOCH_2024_01_01: u64 = 1_704_067_200; // <- digit separators: easier to verify against a known timestamp

fn is_within_limit(size: u64) -> bool {
    size <= MAX_UPLOAD_BYTES
}
```

**Why this way:** grouping digits in threes mirrors how people read large
numbers, which makes a stray or missing digit far more likely to jump out
during review than in one unbroken run — the
[Rust Reference](https://doc.rust-lang.org/reference/tokens.html#integer-literals)
permits `_` anywhere inside a numeric literal specifically to support
this.

## Embedded Rust Notes

**Full support.** Purely cosmetic at compile time — no `std` dependency,
no runtime effect whatsoever.
