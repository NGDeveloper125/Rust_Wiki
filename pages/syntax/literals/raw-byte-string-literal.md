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

## Usage examples

### Representing a file path as raw bytes

```
let path: &[u8] = br"D:\logs\out"; // <- `br"..."`: raw (no escapes) + byte string (&[u8; N])
```

**Restriction:** like any raw literal, if the content itself needs a
`"`, it must be wrapped in matching `#` delimiters — `br#"..."#` — with
enough `#`s to avoid ambiguity.

### Bit manipulation and flags

A byte sequence that's naturally full of backslashes — a Windows-style
path embedded as bytes — reads far better as a raw byte string.

```
// AVOID: every backslash needs doubling, and this is a byte string on top of that
let escaped: &[u8] = b"C:\\Windows\\System32\\drivers\\etc\\hosts";

// PREFER: raw byte string, backslashes are literal bytes, not escapes
let raw: &[u8] = br"C:\Windows\System32\drivers\etc\hosts"; // <- raw byte string literal: no escape processing

assert_eq!(escaped, raw);
```

A raw byte string avoids doubling every backslash in a
byte sequence that's naturally full of them — see
[byte string literal](byte-string-literal.md) for the escape-processing
rules this form opts out of.

## Explanation (Embedded)

Nothing about `br"..."` changes under `#![no_std]` — it produces the
same `'static &[u8; N]` rodata placement as an ordinary byte string, with
no allocator involved. There isn't much genuinely embedded-specific to
add beyond that: the case where this form actually helps — a fixed byte
sequence with several literal backslashes — comes up only occasionally
in embedded code, such as a short binary command sequence that happens
to contain `0x5C` bytes, or a Windows-style path baked into a build-time
constant for a host-side flashing tool. It's a narrow, incidental win
rather than a mainstream embedded idiom; most real firmware binary blobs
are large enough that they come from `include_bytes!` rather than a
hand-written literal at all.

## Usage examples (Embedded)

### A device command sequence that happens to contain literal backslash bytes

```
// AVOID: every 0x5C (backslash) byte needs doubling in an ordinary byte string
let escaped: &[u8] = b"\\CFG\\SET\\baud=115200";

// PREFER: raw byte string, backslash bytes are literal, no escaping needed
let cmd: &[u8] = br"\CFG\SET\baud=115200"; // <- raw byte string literal: backslash bytes taken literally
```
