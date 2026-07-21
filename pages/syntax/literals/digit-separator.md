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

An underscore may be placed almost anywhere within a numeric literal
purely to improve readability, as in `1_000_000` or `0b1010_0101_u8` — it
carries no meaning and does not affect the value.

It's permitted between digits, immediately after a base prefix
(`0x_00FF`), trailing (`10_`), before a type suffix (`1_000_i64`), and in
an exponent. It cannot begin the literal — `_1` lexes as an identifier,
not a number — and it cannot sit immediately after the decimal point of
a float: `1._5` is parsed as field access on `1.`, not a digit separator.

**Not to be confused with:** the wildcard pattern
[`_`](../punctuation/underscore.md) — same character, but a completely
unrelated token whenever it appears outside a numeric literal (a match
arm, a discarded binding, an unused parameter).

## Usage examples

### Grouping digits in a literal for readability

```
let value = 10_000;
//            ^ digit separator: purely cosmetic, does not affect the value
```

**Restriction:** an underscore can't begin the literal (`_1_000` is an
identifier) or follow a float's decimal point directly (`1._5` is field
access, not a separator).

### Numeric computation

A large constant like a byte limit or a Unix timestamp is much easier to
proofread once it's grouped into readable chunks.

```
const MAX_UPLOAD_BYTES: u64 = 10_485_760;   // <- digit separators: 10 MiB, grouped in thousands
const EPOCH_2024_01_01: u64 = 1_704_067_200; // <- digit separators: easier to verify against a known timestamp

fn is_within_limit(size: u64) -> bool {
    size <= MAX_UPLOAD_BYTES
}
```

Grouping digits in threes mirrors how people read large
numbers, which makes a stray or missing digit far more likely to jump out
during review than in one unbroken run — the
[Rust Reference](https://doc.rust-lang.org/reference/tokens.html#integer-literals)
permits `_` anywhere inside a numeric literal specifically to support
this.

## Embedded Rust Notes

**Full support.** Purely cosmetic at compile time — no `std` dependency,
no runtime effect whatsoever.
