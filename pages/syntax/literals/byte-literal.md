---
title: "Byte literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [char-literal, byte-string-literal]
see_also: [char-literal]
---

## Explanation

A byte literal, `b'H'`, produces a `u8` — the ASCII code point of the
character between the quotes — rather than a `char`:

```
let b: u8 = b'H'; // 72
```

Only ASCII characters (code points 0–127) are legal inside a byte
literal; anything requiring Unicode beyond ASCII is a compile error,
since a single `u8` can't represent it. This is distinct from a
[character literal](char-literal.md) (`'H'`), which produces a full
Unicode `char` (4 bytes) instead of a `u8`.

## Embedded Rust Notes

**Full support.** No `std` dependency — commonly used for protocol magic
bytes and framing constants in embedded communication code.
