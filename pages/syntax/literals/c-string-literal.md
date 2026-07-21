---
title: "C string literal"
kind: literal
embedded_support: full
groups: [Basics, "Memory & Unsafe / FFI"]
related_concepts: [FFI]
related_syntax: [string-literal, raw-c-string-literal]
see_also: [string-literal]
---

## Explanation

`c"hello"` produces a `&CStr` — a nul-terminated, C-compatible string
reference — instead of `&str`.

This exists specifically to make passing string constants across an FFI
boundary to C code ergonomic: the compiler appends the terminating `\0`
and produces the right type directly, instead of requiring
`CString::new("hello").unwrap()` at runtime for what is really a
compile-time-known constant. Unlike a byte string, a `c"..."` literal
still accepts full Unicode content (encoded as UTF-8, same as a normal
string literal) — the constraint is "no embedded nul bytes," not
"ASCII only."

## Usage examples

### Producing a nul-terminated C string

```
let name: &std::ffi::CStr = c"sensor01"; // <- c-string literal: produces `&CStr`, nul-terminated
```

The content cannot contain an embedded `\0` — a C
string is nul-terminated, so an interior null byte is a compile error.

### Working with text

Preparing a `CStr` constant that will eventually be handed to a C API is
just a matter of writing the literal and holding onto it — the FFI call
itself is a separate concern, not shown here.

```
use std::ffi::CStr;

// A device name a C driver expects as a nul-terminated string.
const DEVICE_NAME: &CStr = c"sensor-hub-01"; // <- c-string literal: nul terminator added at compile time

fn device_name() -> &'static CStr {
    DEVICE_NAME // ready to pass to a C function expecting `*const c_char` -- the call itself is out of scope here
}
```

A `c"..."` literal produces its nul-terminated bytes at
compile time, so a fixed constant like this never needs the fallible,
allocating `CString::new(...).unwrap()` path at runtime — it hands back a
`&'static CStr` directly (see the
[std docs for `CStr`](https://doc.rust-lang.org/std/ffi/struct.CStr.html)).
Note `c"..."` literals require Rust 1.77+ and edition 2021 or later.

## Explanation (Embedded)

`c"..."` is unaffected by `#![no_std]` — `CStr` lives in `core::ffi`
(re-exported as `std::ffi::CStr` on hosted targets), so the exact same
literal and the exact same type are available on a bare-metal target.
This is precisely where a `c"..."` literal earns its keep in embedded
work: wrapping a vendor's C HAL or SDK — an ST HAL, an ESP-IDF header, a
vendor RTOS's C API — means calling functions that expect a
`*const c_char`, and a compile-time `CStr` constant avoids ever needing
an allocator (`CString::new(...)`) just to hand a fixed name or
identifier string across that FFI boundary.

## Usage examples (Embedded)

### Naming a peripheral for a vendor C HAL call

```
use core::ffi::{c_char, CStr};

extern "C" {
    fn hal_register_peripheral(name: *const c_char) -> i32;
}

const PERIPHERAL_NAME: &CStr = c"uart0"; // <- c-string literal: nul-terminated, ready for the C HAL's `*const c_char` parameter

fn register_uart() -> i32 {
    unsafe { hal_register_peripheral(PERIPHERAL_NAME.as_ptr()) }
}
```
