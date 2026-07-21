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
a fixed-size array of bytes — rather than a `&str`.

Like a byte literal, the *character* content must be ASCII — but `\xHH`
byte escapes reach the full `0x00`–`0xFF` range (`b"\x89PNG"` is legal,
with byte `0x89` = 137). What you can't write directly is a non-ASCII
*character* like `b"café"`. Useful for binary protocol constants or
magic-number byte sequences where you specifically want raw bytes, not
validated UTF-8 text.

## Usage examples

### Producing a fixed-size byte array

```
let magic: &[u8; 3] = b"GIF"; // <- byte string literal: produces `&[u8; N]`, not `&str`
```

*Character* content must be ASCII (`b"café"` is an
error), but `\xHH` escapes reach any byte `0x00`–`0xFF` — which is how a
byte string still expresses non-ASCII bytes like `b"\xFF"`.

### Bit manipulation and flags

A multi-byte file-format signature — like PNG's 8-byte magic number —
reads and compares cleanly as one byte string literal.

```
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n"; // <- byte string literal: the exact 8-byte PNG magic number

fn looks_like_png(data: &[u8]) -> bool {
    data.starts_with(PNG_SIGNATURE)
}
```

Writing the whole magic number as one `b"..."` literal
keeps the exact byte sequence readable and comparable in a single place,
checked against real input with
[`slice::starts_with`](https://doc.rust-lang.org/std/primitive.slice.html#method.starts_with).

### Validating input

Checking a fixed byte signature before attempting to parse the rest of a
stream turns "is this even the right format" into one cheap check up
front.

```
fn parse_gzip_header(data: &[u8]) -> Result<(), &'static str> {
    const GZIP_MAGIC: &[u8; 2] = b"\x1f\x8b"; // <- byte string literal: fixed 2-byte signature to validate against

    if !data.starts_with(GZIP_MAGIC) {
        return Err("not a gzip stream: bad magic bytes");
    }
    Ok(())
}
```

Validating the signature bytes before doing any real
parsing work turns a malformed-input case into one obviously-correct
check up front, instead of discovering the mismatch deep inside a
partially-completed parse — rejecting bad input at the boundary keeps the
rest of the parser working only with data it has already vetted.

## Explanation (Embedded)

A byte string literal is exactly as fully supported under `#![no_std]` as
under `std` — `b"..."` still produces a `'static &[u8; N]` baked into the
binary's rodata section at compile time, with no allocator involved at
any point. That makes it the natural way to write a fixed protocol frame
or firmware header in embedded code: a UART command frame, a sensor's
request/acknowledge byte sequence, or a bootloader's magic-number header
check are all naturally byte data rather than valid UTF-8 text, and
`b"..."` lets that data be written as readable ASCII-with-escapes instead
of a bare `[u8; N]` array of numeric literals. The same ASCII-plus-`\xHH`-
escape rule from the classic explanation carries over unchanged — a
non-ASCII framing byte like `0xAA` is written `\xAA` exactly as it would
be on a hosted target.

## Usage examples (Embedded)

### Framing a UART command

```
const REQUEST_TEMPERATURE: &[u8; 4] = b"\x02T?\x03"; // <- byte string literal: STX, command, ETX framing bytes

fn send_command(uart: &mut impl embedded_hal::serial::Write<u8>, cmd: &[u8]) {
    for &byte in cmd {
        nb::block!(uart.write(byte)).ok();
    }
}
```

### Validating a firmware image header before boot

```
fn is_valid_firmware(image: &[u8]) -> bool {
    const FIRMWARE_MAGIC: &[u8; 4] = b"FWv2"; // <- byte string literal: bootloader magic-number header
    image.starts_with(FIRMWARE_MAGIC)
}
```
