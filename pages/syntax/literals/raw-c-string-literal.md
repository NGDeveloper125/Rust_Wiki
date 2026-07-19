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

## Best practices & deeper information

### Scenario: Working with text

A path handed to a C API needs both nul-termination and, often, literal
backslashes — `cr"..."` prepares that value without pretending to make
the FFI call itself.

```
use std::ffi::CStr;

// A firmware image path for a C loader that expects a nul-terminated C string.
const FIRMWARE_PATH: &CStr = cr"C:\firmware\images\boot.bin"; // <- raw c-string literal: no escapes AND nul-terminated

fn firmware_path() -> &'static CStr {
    FIRMWARE_PATH // ready to pass across an FFI boundary -- the call itself is out of scope here
}
```

**Why this way:** combining raw and C-string semantics avoids the double
burden of escaping every backslash *and* manually appending a nul
terminator — see [C string literal](c-string-literal.md) for the
nul-termination behavior this form inherits unchanged.

## Embedded Rust Notes

**Full support** — same `core::ffi::CStr` basis as
[C string literal](c-string-literal.md), no `std` dependency.
