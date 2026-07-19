---
title: "Octal integer literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["Numeric types & overflow behavior"]
related_syntax: [integer-suffixes, digit-separator]
see_also: [integer-decimal, integer-hexadecimal, integer-binary]
---

## Explanation

A base-8 integer literal, prefixed with `0o`:

```
let permissions = 0o755;
```

Note the letter `o`, not a digit `0` — unlike C's ambiguous leading-zero
octal notation (`0755`), Rust requires the explicit `0o` prefix, so a
literal like `0755` is just decimal 755, never accidentally
misinterpreted as octal.

## Basic usage example

```
let mode = 0o644; // <- `0o` prefix marks a base-8 (octal) integer literal
```

**Restriction:** only digits `0`–`7` are legal after the `0o` prefix —
`0o8` is a compile error.

## Embedded Rust Notes

**Full support.** No `std` dependency; rarely used in embedded code
specifically (hex is the near-universal convention for addresses and
masks), but fully available.
