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
character between the quotes — rather than a `char`.

The *character* form must be ASCII (code points 0–127) — a non-ASCII
character like `b'é'` is a compile error. To reach the rest of the
`u8` range, use a byte escape: `b'\xFF'` (255) is perfectly legal, since
`\xHH` covers the full `0x00`–`0xFF`. This is distinct from a
[character literal](char-literal.md) (`'H'`), which produces a full
Unicode `char` (4 bytes) instead of a `u8`.

## Usage examples

### Producing a u8 byte value

```
let byte: u8 = b'A'; // <- byte literal: produces a `u8` (65), not a `char`
```

The *character* form is ASCII-only (`b'é'` is a compile
error), but a `\xHH` byte escape reaches the full `0x00`–`0xFF` range
(`b'\xFF'` = 255).

### Bit manipulation and flags

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

Matching against `b'\x7E'` directly reads as "this
specific marker byte," clearer at the call site than comparing against a
bare numeric constant like `126` — byte literals are valid patterns, as
the [Reference's patterns chapter](https://doc.rust-lang.org/reference/patterns.html)
describes for literal patterns generally.

## Embedded Rust Notes

**Full support.** No `std` dependency — commonly used for protocol magic
bytes and framing constants in embedded communication code.
