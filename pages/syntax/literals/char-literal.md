---
title: "Character literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [byte-literal, escape-sequences]
see_also: [byte-literal]
---

## Explanation

A single-quoted character literal produces a `char` — a full Unicode
scalar value, always 4 bytes, not a single byte:

```
let c: char = 'H';
let emoji: char = '🦀';
```

This is a common surprise for newcomers: `char` in Rust is not "one byte"
the way `char` is in C — it represents any Unicode scalar value
(excluding surrogate-pair halves), which is why iterating a `String`
byte-by-byte and iterating it `char`-by-char (`.chars()`) can give very
different results for non-ASCII text. See
[byte literal](byte-literal.md) for the ASCII-only, single-byte
equivalent (`b'H'`).

## Embedded Rust Notes

**Full support.** `char` is a `core` primitive — no `std` dependency,
though its 4-byte size is worth remembering on very memory-constrained
targets where a raw [byte literal](byte-literal.md) (`u8`) is often the
more appropriate choice for protocol/text work that's ASCII-only anyway.
