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
byte string), as in `br"C:\data\raw"`.

Useful when a fixed byte sequence contains literal backslashes you don't
want interpreted as escapes.

## Basic usage example

```
let path: &[u8] = br"D:\logs\out"; // <- `br"..."`: raw (no escapes) + byte string (&[u8; N])
```

**Restriction:** like any raw literal, if the content itself needs a
`"`, it must be wrapped in matching `#` delimiters — `br#"..."#` — with
enough `#`s to avoid ambiguity.

## Best practices & deeper information

### Scenario: Bit manipulation and flags

A byte sequence that's naturally full of backslashes — a Windows-style
path embedded as bytes — reads far better as a raw byte string.

```
// AVOID: every backslash needs doubling, and this is a byte string on top of that
let escaped: &[u8] = b"C:\\Windows\\System32\\drivers\\etc\\hosts";

// PREFER: raw byte string, backslashes are literal bytes, not escapes
let raw: &[u8] = br"C:\Windows\System32\drivers\etc\hosts"; // <- raw byte string literal: no escape processing

assert_eq!(escaped, raw);
```

**Why this way:** a raw byte string avoids doubling every backslash in a
byte sequence that's naturally full of them — see
[byte string literal](byte-string-literal.md) for the escape-processing
rules this form opts out of.

## Embedded Rust Notes

**Full support.** No `std` dependency, no allocation required.
