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

## Basic usage example

```
let byte: u8 = b'A'; // <- byte literal: produces a `u8` (65), not a `char`
```

**Restriction:** only ASCII characters (code points 0–127) are legal
inside a byte literal — anything requiring Unicode beyond ASCII is a
compile error.

## Best practices & deeper information

### Scenario: Bit manipulation and flags

Matching a single magic byte in a protocol header reads clearest against
a byte literal, right inside the match pattern.

```
fn parse_frame(data: &[u8]) -> Option<&[u8]> {
    match data.first() {
        Some(&b'\x7E') => Some(&data[1..]), // <- byte literal: matches the frame's single start-of-frame marker
        _ => None,
    }
}

assert_eq!(parse_frame(&[0x7E, 1, 2, 3]), Some(&[1u8, 2, 3][..]));
```

**Why this way:** matching against `b'\x7E'` directly reads as "this
specific marker byte," clearer at the call site than comparing against a
bare numeric constant like `126` — see the
[std primitive `u8` docs](https://doc.rust-lang.org/std/primitive.u8.html)
for byte-oriented pattern matching in general.

## Embedded Rust Notes

**Full support.** No `std` dependency — commonly used for protocol magic
bytes and framing constants in embedded communication code.
