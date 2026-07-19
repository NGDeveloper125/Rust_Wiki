---
title: "Raw C string literal"
kind: literal
embedded_support: full
groups: [Basics, "Memory & Unsafe / FFI"]
related_concepts: [FFI]
related_syntax: [c-string-literal, raw-string-literal]
see_also: [c-string-literal]
---

## Explanation

`cr"..."` (or `cr#"..."#`) combines the C-string and raw-string forms: a
`&CStr` result with no escape processing.

```
let s: &std::ffi::CStr = cr"C:\path\to\thing";
```

Useful for FFI constants that both need the C-compatible nul-terminated
representation and contain literal backslashes.

## Basic usage example

```
let path: &std::ffi::CStr = cr"C:\firmware\boot"; // <- `cr"..."`: raw (no escapes) C string (&CStr)
```

**Restriction:** the content still cannot contain an embedded NUL byte,
and matching `#` delimiters (`cr#"..."#`) are required if the text
itself contains a `"`.

## Embedded Rust Notes

**Full support** — same `core::ffi::CStr` basis as
[C string literal](c-string-literal.md), no `std` dependency.
