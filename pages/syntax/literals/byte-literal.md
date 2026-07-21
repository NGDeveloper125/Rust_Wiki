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

## Explanation (Embedded)

A byte literal means exactly the same thing under `#![no_std]` — it's
parsed and typed at compile time, producing a plain `u8` value or pattern,
with no allocator or runtime involved. Where it earns its keep in
embedded code is UART and protocol handling: firmware reading raw bytes
off a serial peripheral has no `String`/`str` layer standing between it
and the wire, so individual bytes get compared and matched directly
against byte literals for terminators, ACK/NAK codes, and command
characters. It's the same job a byte literal does parsing a binary
format on a hosted system — just proportionally more central in embedded
code, where raw byte streams are often the *only* text-shaped thing the
firmware ever sees.

## Usage examples (Embedded)

### Reading a UART byte stream up to a line terminator

```
use embedded_hal::serial::Read;
use nb::block;

fn read_line<S: Read<u8>>(uart: &mut S, buf: &mut [u8; 64]) -> usize {
    let mut n = 0;
    while let Ok(byte) = block!(uart.read()) {
        if byte == b'\n' { // <- byte literal: UART line terminator, compared against a raw received byte
            break;
        }
        if n < buf.len() {
            buf[n] = byte;
            n += 1;
        }
    }
    n
}
```

### Recognizing an AT-command response

```
fn is_ack(response: &[u8]) -> bool {
    matches!(response, [b'O', b'K', ..]) // <- byte literals: matching an "OK" modem/AT-command reply byte by byte
}
```
