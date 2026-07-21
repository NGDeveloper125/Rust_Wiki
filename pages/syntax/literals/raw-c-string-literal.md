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
`&CStr` result with no escape processing, as in `cr"C:\path\to\thing"`.

Useful for FFI constants that both need the C-compatible nul-terminated
representation and contain literal backslashes.

## Usage examples

### Representing a firmware path for FFI

```
let path: &std::ffi::CStr = cr"C:\firmware\boot"; // <- `cr"..."`: raw (no escapes) C string (&CStr)
```

**Restriction:** the content still cannot contain an embedded NUL byte,
and matching `#` delimiters (`cr#"..."#`) are required if the text
itself contains a `"`.

### Working with text

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

Combining raw and C-string semantics avoids the double
burden of escaping every backslash *and* manually appending a nul
terminator — see [C string literal](c-string-literal.md) for the
nul-termination behavior this form inherits unchanged. Like `c"..."`,
`cr"..."` requires Rust 1.77+ and edition 2021 or later.

## Explanation (Embedded)

`cr"..."` inherits both halves unchanged under `#![no_std]`: the
`core::ffi::CStr` result from a C-string literal, and the raw,
no-escape-processing text from a raw string. That combination is
genuinely useful in embedded FFI work whenever the fixed, nul-terminated
string handed to a vendor C HAL or SDK function itself contains
backslashes — a Windows-style path baked into a host-side
firmware-flashing tool that calls into a vendor's C flashing library, or
a fixed pattern string passed to a C library's matching function —
without needing to double every backslash on top of remembering the nul
terminator.

## Usage examples (Embedded)

### Passing a firmware image path to a vendor C flashing SDK

```
use core::ffi::{c_char, CStr};

extern "C" {
    fn vendor_flash_load_image(path: *const c_char) -> i32;
}

const IMAGE_PATH: &CStr = cr"C:\firmware\images\app.bin"; // <- `cr"..."`: raw (no escapes) + nul-terminated, ready for FFI

fn flash_image() -> i32 {
    unsafe { vendor_flash_load_image(IMAGE_PATH.as_ptr()) }
}
```
