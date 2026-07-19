---
title: "<<"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["<<=", ">>"]
see_also: [">>"]
---

## Explanation

`<<` is the left-shift operator, overloadable via `std::ops::Shl`:

```
let x = 1u8 << 3; // 8
```

Shifting by an amount greater than or equal to the type's bit width is a
panic in debug builds (and unspecified/masked behavior to guard against in
release — check `checked_shl`/`wrapping_shl` for defined behavior at the
boundary). `<<` on integers is a pure bit-shift, unrelated to C++'s
overload of `<<` for stream output — Rust uses `{}`/`write!` and the
`Display`/`Debug` traits for formatting instead.

## Basic usage example

```
let x = 1u8 << 3; // <- `<<` shifts the bits of `1u8` left by 3
```

**Restriction:** shifting by an amount greater than or equal to the
type's bit width panics in debug builds; use `checked_shl`/`wrapping_shl`
for defined behavior at the boundary.

## Embedded Rust Notes

**Full support.** `Shl` lives in `core::ops` — bit shifts are used
constantly in embedded code to construct register masks (`1 << pin_num`).
