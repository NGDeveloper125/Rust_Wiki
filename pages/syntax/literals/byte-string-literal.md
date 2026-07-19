---
title: "Byte string literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [string-literal, byte-literal, raw-byte-string-literal]
see_also: [string-literal]
---

## Explanation

A byte string literal, `b"hello"`, produces a `&[u8; N]` — a reference to
a fixed-size array of bytes — rather than a `&str`:

```
let bytes: &[u8; 5] = b"hello";
```

Like a byte literal, only ASCII content is allowed (no arbitrary Unicode
escapes beyond ASCII/byte escapes). Useful for binary protocol constants
or magic-number byte sequences where you specifically want raw bytes, not
validated UTF-8 text.

## Embedded Rust Notes

**Full support.** Like a string literal, a byte string lives in the
binary's read-only data with no allocation — ideal for protocol frame
templates and lookup tables in embedded code.
