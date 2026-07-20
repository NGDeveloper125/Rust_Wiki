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
depends on the target platform) and are the required type for array
indices and lengths.

## Basic usage example

```
let port = 8080u16; // <- `u16` suffix pins the literal's type explicitly
```

**Restriction:** the literal's value must fit within the suffixed
type's range — `300u8` is a compile error since `u8` maxes out at 255.

## Best practices & deeper information

### Scenario: Numeric computation

Without a suffix, a bare integer literal defaults to `i32` — which is
wrong the moment the value needs to be wider, and can fail to compile
outright (`1 << 40` overflows `i32` at compile time).

```
fn file_offset(block: u64, block_size: u64) -> u64 {
    block * block_size
}

// AVOID: bare literal defaults to i32, then needs a cast at the call site
let bad_size = 4096;
let offset_bad = file_offset(3, bad_size as u64);

// PREFER: suffix the literal once, right where inference would otherwise guess i32
let block_size = 4096u64; // <- `u64` suffix: matches `file_offset`'s parameter type, no cast needed
let offset = file_offset(3, block_size);
```

**Why this way:** suffixing the literal at its definition removes the
need for an `as` cast later — casts are exactly where
[Clippy's cast-truncation lints](https://rust-lang.github.io/rust-clippy/master/index.html#cast_possible_truncation)
look for accidental narrowing, so pinning the type once, early, avoids
the whole class of bug.

### Scenario: Designing a public API

A public constant's type is part of its contract with downstream crates —
suffixing the literal documents the exact intended width at the
definition site.

```
/// Maximum payload size accepted by this protocol version, in bytes.
pub const MAX_PAYLOAD_BYTES: u16 = 1024u16; // <- suffix pins the exact width consumers can rely on

pub fn check_payload(len: usize) -> bool {
    len <= MAX_PAYLOAD_BYTES as usize
}
```

**Why this way:** matching the literal's suffix to the constant's
declared type keeps the intended width visible even if the value is ever
copied into a doc example or another context without the surrounding
type annotation — the kind of predictability the
[API Guidelines' predictability chapter](https://rust-lang.github.io/api-guidelines/predictability.html)
asks public items to favor.

## Embedded Rust Notes

**Full support.** No `std` dependency. `usize`/`isize` are still
pointer-sized on embedded targets, but that width is often much narrower
than on a desktop host (commonly 32-bit, sometimes 16-bit on very small
parts) — don't assume `usize` means "at least 64 bits" in embedded code.
