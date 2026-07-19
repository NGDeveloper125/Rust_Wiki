---
title: "Raw byte string literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [byte-string-literal, raw-string-literal]
see_also: [byte-string-literal]
---

## Explanation

`br"..."` (optionally with `#` delimiters, `br#"..."#`) combines the two:
no escape processing (like a raw string) and a `&[u8; N]` result (like a
byte string).

```
let pattern: &[u8] = br"C:\data\raw";
```

Useful when a fixed byte sequence contains literal backslashes you don't
want interpreted as escapes.

## Embedded Rust Notes

**Full support.** No `std` dependency, no allocation required.
