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
`u64`, `i64`, `u128`, `i128`, `usize`, `isize` — for example `255u8` or
`10_i64`.

An underscore may appear between the digits and the suffix purely for
readability (`10_i64` and `10i64` are identical). Without a suffix, the
compiler infers the type from context — how the value is used, what it's
assigned to, what function receives it — and only falls back to `i32` if
nothing else constrains it. `usize`/`isize` are pointer-sized (their width
depends on the target platform) and are the required type for array
indices and lengths.

## Usage examples

### Pinning a port number's type

```
let port = 8080u16; // <- `u16` suffix pins the literal's type explicitly
```

**Restriction:** the literal's value must fit within the suffixed
type's range — `300u8` is a compile error since `u8` maxes out at 255.

### Numeric computation

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

Suffixing the literal at its definition removes the
need for an `as` cast later — casts are exactly where
[Clippy's cast-truncation lints](https://rust-lang.github.io/rust-clippy/master/index.html#cast_possible_truncation)
look for accidental narrowing, so pinning the type once, early, avoids
the whole class of bug.

### Designing a public API

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

Matching the literal's suffix to the constant's
declared type keeps the intended width visible even if the value is ever
copied into a doc example or another context without the surrounding
type annotation — the kind of predictability the
[API Guidelines' predictability chapter](https://rust-lang.github.io/api-guidelines/predictability.html)
asks public items to favor.

## Explanation (Embedded)

A type suffix pins a literal's width at compile time exactly the same
way under `#![no_std]`, but *which* width you reach for matters more in
embedded code than it typically does on a desktop host. In hosted code,
`i32`/`usize` are usually fine defaults for a value with no strong width
requirement of its own; in embedded code, a value very often represents
a specific hardware field — an 8-bit GPIO data register, a 12-bit ADC
result held in a `u16`, a 32-bit memory-mapped register — whose width is
fixed by the chip, not a matter of taste. Suffixing a constant to that
exact width turns a too-large value into a compile error at its
definition site, instead of a silent truncation discovered later through
an `as` cast at the point the constant is finally used. (Separately,
`usize`/`isize` are still pointer-sized on embedded targets, but that
width is often much narrower than on a desktop host — commonly 32-bit,
sometimes 16-bit on very small parts — so it's worth not assuming
`usize` means "at least 64 bits" in embedded code.)

## Usage examples (Embedded)

### Pinning a constant to the exact width of a peripheral register

```
const ADC_MAX: u16 = 4095u16; // <- `u16` suffix: matches the ADC's 12-bit result register width exactly
```

### Catching an out-of-range register value at compile time

```
const GPIO_PIN_MASK: u8 = 0b1111_1111u8; // <- `u8` suffix: this MCU's GPIO data register is only 8 bits wide
// const BAD_MASK: u8 = 300u8; // would be a compile error: 300 doesn't fit in a u8
```
