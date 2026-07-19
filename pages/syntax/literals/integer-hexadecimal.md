---
title: "Hexadecimal integer literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["Numeric types & overflow behavior"]
related_syntax: [integer-suffixes, digit-separator]
see_also: [integer-decimal, integer-octal, integer-binary]
---

## Explanation

A base-16 integer literal, prefixed with `0x`:

```
let mask = 0xFF_u8;
let color = 0x1a2b3c;
```

Digits `a`–`f` may be upper- or lower-case, and can be mixed within the
same literal (though consistent casing is the usual style convention).
Underscores are allowed between digits, including immediately after the
`0x` prefix. Like all integer literals, an optional type suffix
(`0xffu8`) can pin the type directly.

## Basic usage example

```
let addr = 0x2000; // <- `0x` prefix marks a base-16 (hexadecimal) integer literal
```

**Restriction:** only digits `0`–`9` and letters `a`–`f`/`A`–`F` are
legal after the `0x` prefix.

## Embedded Rust Notes

**Full support.** Hex literals are the conventional way to write register
addresses and bitmasks in embedded code (`0x4001_0000`, `0xFF`) — no
`std` dependency.
