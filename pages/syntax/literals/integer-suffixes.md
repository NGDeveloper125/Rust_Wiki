---
title: "Integer suffixes"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["Numeric types & overflow behavior", "Type inference"]
related_syntax: [integer-decimal, float-suffixes]
see_also: [integer-decimal]
---

## Explanation

A type suffix pins an integer literal's type explicitly, written directly
after the digits with no space: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`,
`u64`, `i64`, `u128`, `i128`, `usize`, `isize`.

```
let a = 255u8;
let b = 10_i64;
```

An underscore may appear between the digits and the suffix purely for
readability (`10_i64` and `10i64` are identical). Without a suffix, the
compiler infers the type from context — how the value is used, what it's
assigned to, what function receives it — and only falls back to `i32` if
nothing else constrains it. `usize`/`isize` are pointer-sized (their width
depends on the target platplatform) and are the required type for array
indices and lengths.

## Basic usage example

```
let port = 8080u16; // <- `u16` suffix pins the literal's type explicitly
```

**Restriction:** the literal's value must fit within the suffixed
type's range — `300u8` is a compile error since `u8` maxes out at 255.

## Embedded Rust Notes

**Full support.** No `std` dependency. `usize`/`isize` are still
pointer-sized on embedded targets, but that width is often much narrower
than on a desktop host (commonly 32-bit, sometimes 16-bit on very small
parts) — don't assume `usize` means "at least 64 bits" in embedded code.
