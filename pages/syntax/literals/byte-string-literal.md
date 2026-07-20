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

## Basic usage example

```
let magic: &[u8; 3] = b"GIF"; // <- byte string literal: produces `&[u8; N]`, not `&str`
```

**Restriction:** *character* content must be ASCII (`b"café"` is an
error), but `\xHH` escapes reach any byte `0x00`–`0xFF` — which is how a
byte string still expresses non-ASCII bytes like `b"\xFF"`.

## Best practices & deeper information

### Scenario: Bit manipulation and flags

A multi-byte file-format signature — like PNG's 8-byte magic number —
reads and compares cleanly as one byte string literal.

```
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n"; // <- byte string literal: the exact 8-byte PNG magic number

fn looks_like_png(data: &[u8]) -> bool {
    data.starts_with(PNG_SIGNATURE)
}
```

**Why this way:** writing the whole magic number as one `b"..."` literal
keeps the exact byte sequence readable and comparable in a single place,
checked against real input with
[`slice::starts_with`](https://doc.rust-lang.org/std/primitive.slice.html#method.starts_with).

### Scenario: Validating input

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

**Why this way:** validating the signature bytes before doing any real
parsing work turns a malformed-input case into one obviously-correct
check up front, instead of discovering the mismatch deep inside a
partially-completed parse — rejecting bad input at the boundary keeps the
rest of the parser working only with data it has already vetted.

## Embedded Rust Notes

**Full support.** Like a string literal, a byte string lives in the
binary's read-only data with no allocation — ideal for protocol frame
templates and lookup tables in embedded code.
