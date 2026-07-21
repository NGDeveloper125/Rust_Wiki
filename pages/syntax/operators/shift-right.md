---
title: ">>"
kind: operator
embedded_support: full
groups: [Bitwise, Basics]
related_concepts: [Operator overloading]
related_syntax: [">>=", "<<"]
see_also: ["<<"]
---

## Explanation

`>>` is the right-shift operator, overloadable via `std::ops::Shr`.

For unsigned integers this is always a logical shift (zero-fill); for
signed integers it's an arithmetic shift (sign-bit-fill), matching the
type's own notion of sign. See [`<<`](shift-left.md) for the shared notes
on out-of-range shift amounts.

## Basic usage example

```
let x = 8u8 >> 3; // <- `>>` shifts the bits of `8u8` right by 3
```

**Restriction:** as with `<<`, shifting by an amount greater than or
equal to the type's bit width panics in debug builds; with overflow
checks off, the shift amount is masked to the bit width (defined
behavior, but usually not what you meant).

## Best practices & deeper information

### Scenario: Bit manipulation and flags

Extracting a specific field from a packed byte combines `>>` (move the
field down to bit 0) with `&` (mask off everything else) — a pairing
that shows up constantly when decoding protocol or register bytes.

```
// A status byte: bits 7-4 = error code, bits 3-0 = flags
let status: u8 = 0b0101_0011;

let error_code = status >> 4; // <- `>>` moves the high nibble down to bit 0
let flags = status & 0b1111;   // low nibble, no shift needed

assert_eq!(error_code, 0b0101);
assert_eq!(flags, 0b0011);
```

**Why this way:** shifting before masking (or masking before shifting,
for a low field) is the standard field-extraction idiom for packed data —
[Rust by Example](https://doc.rust-lang.org/rust-by-example/primitives/literals.html)
shows the basic shift and mask operators these idioms build on.

## Embedded Rust Notes

**Full support.** `Shr` lives in `core::ops` — no `std` dependency.
