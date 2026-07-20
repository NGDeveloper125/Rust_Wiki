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

A base-8 integer literal, prefixed with `0o`, as in `0o755`.

Note the letter `o`, not a digit `0` — unlike C's ambiguous leading-zero
octal notation (`0755`), Rust requires the explicit `0o` prefix, so a
literal like `0755` is just decimal 755, never accidentally
misinterpreted as octal.

## Basic usage example

```
let mode = 0o644; // <- `0o` prefix marks a base-8 (octal) integer literal
```

**Restriction:** only *digits* `0`–`7` may appear after the `0o` prefix —
`0o8` is a compile error (underscores and a type suffix like `0o644_u16`
are still allowed).

## Best practices & deeper information

### Scenario: Bit manipulation and flags

Unix-style file permission bits are conventionally written and read in
octal, since each digit maps exactly onto one `rwx` triplet.

```
let mode: u32 = 0o755; // <- octal literal: owner rwx, group rx, other rx — matches `ls -l` directly

let owner_bits = (mode >> 6) & 0o7;
let group_bits = (mode >> 3) & 0o7;
let other_bits = mode & 0o7;

assert_eq!((owner_bits, group_bits, other_bits), (0o7, 0o5, 0o5));
```

**Why this way:** Unix permission tooling (`chmod 755`, `ls -l`) is itself
built around octal because each digit is one 3-bit `rwx` group — a
mapping that's lost if the same mode is written in hex or decimal; for
masks that aren't naturally 3-bit-grouped, hex is the more common
convention — see [integer-hexadecimal](integer-hexadecimal.md).

## Embedded Rust Notes

**Full support.** No `std` dependency; rarely used in embedded code
specifically (hex is the near-universal convention for addresses and
masks), but fully available.
