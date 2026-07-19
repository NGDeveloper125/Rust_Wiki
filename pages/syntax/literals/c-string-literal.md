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
reference — instead of `&str`:

```
let s: &std::ffi::CStr = c"hello";
```

This exists specifically to make passing string constants across an FFI
boundary to C code ergonomic: the compiler appends the terminating `\0`
and produces the right type directly, instead of requiring
`CString::new("hello").unwrap()` at runtime for what is really a
compile-time-known constant. Unlike a byte string, a `c"..."` literal
still accepts full Unicode content (encoded as UTF-8, same as a normal
string literal) — the constraint is "no embedded nul bytes," not
"ASCII only."

## Basic usage example

```
let name: &std::ffi::CStr = c"sensor01"; // <- c-string literal: produces `&CStr`, nul-terminated
```

**Restriction:** the content cannot contain an embedded `\0` — a C
string is nul-terminated, so an interior null byte is a compile error.

## Embedded Rust Notes

**Full support** — `CStr` lives in `core::ffi`, not `std::ffi`, which
matters a great deal for embedded: calling into a vendor C HAL or RTOS
API is one of the most common reasons to reach for a nul-terminated
string in embedded Rust at all.
