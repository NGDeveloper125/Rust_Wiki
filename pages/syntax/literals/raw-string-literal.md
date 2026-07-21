---
title: "Raw string literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["String vs &str"]
related_syntax: [string-literal, escape-sequences]
see_also: [string-literal]
---

## Explanation

A raw string literal disables escape processing entirely — every
character between the quotes is taken literally, including backslashes,
as in `r"C:\Users\name"` or a regex pattern like `r"\d+\.\d+"`.

When the string itself needs to contain a `"`, wrap it in matching `#`
delimiters — `r#"..."#` — and use as many `#` as needed to avoid ambiguity
with any `#` sequences inside the content itself (`r##"contains "# inside"##`).
Like a normal string literal, the result type is `&str`; the only
difference is how the literal's *source text* is interpreted, not the
resulting type.

## Usage examples

### Representing a Windows-style file path

```
let path = r"C:\temp\file"; // <- `r"..."`: raw string, backslashes are literal, not escapes
```

**Restriction:** `#` delimiters must be balanced and matched —
`r#"..."#` needs the same number of `#` on both sides, chosen high
enough to avoid ambiguity with any `#` sequences in the content.

### Working with text

A Windows path written as a normal string literal needs every backslash
doubled — a raw string sidesteps that entirely.

```
// AVOID: every backslash must be doubled, easy to miscount
let escaped = "C:\\Users\\alice\\AppData\\config.toml";

// PREFER: raw string, backslashes are literal, no escaping needed
let path = r"C:\Users\alice\AppData\config.toml"; // <- raw string literal: backslashes taken literally

assert_eq!(escaped, path);
```

A raw string removes the need to double every
backslash, which for a Windows path or a regex pattern (`r"\d+\.\d+"`)
quickly becomes hard to both write correctly and review; see
[string literal](string-literal.md) for the general escape-processing
rules a raw string opts out of.

## Explanation (Embedded)

A raw string is unaffected by `#![no_std]` in exactly the same way an
ordinary string literal is — it's still `&'static str` rodata, resolved
entirely at compile time, no allocator involved. There isn't much
genuinely new to say for embedded specifically: the backslash-heavy
content a raw string is built for — regex patterns, Windows-style paths
— shows up on the host-tooling side of embedded work (build scripts,
flashing utilities, config parsers) more often than in on-target
firmware code, which tends to deal in byte buffers rather than
backslash-laden text. Where it does help on-target is the same
readability win as on desktop: a fixed pattern or path string with
several backslashes is easier to write and review without doubling each
one.

## Usage examples (Embedded)

### A path constant for a host-side firmware-flashing tool

```
// AVOID: every backslash must be doubled, easy to miscount
let firmware_dir = "C:\\Users\\dev\\firmware\\builds";

// PREFER: raw string, backslashes are literal, no escaping needed
let firmware_dir = r"C:\Users\dev\firmware\builds"; // <- raw string literal: backslashes taken literally

assert_eq!(firmware_dir.len(), 27);
```
