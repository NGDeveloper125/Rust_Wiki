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

## Usage examples

### Right-shifting the bits of an integer

```
let x = 8u8 >> 3; // <- `>>` shifts the bits of `8u8` right by 3
```

**Restriction:** as with `<<`, shifting by an amount greater than or
equal to the type's bit width panics in debug builds; with overflow
checks off, the shift amount is masked to the bit width (defined
behavior, but usually not what you meant).

### Bit manipulation and flags

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

Shifting before masking (or masking before shifting,
for a low field) is the standard field-extraction idiom for packed data —
[Rust by Example](https://doc.rust-lang.org/rust-by-example/primitives/literals.html)
shows the basic shift and mask operators these idioms build on.

## Explanation (Embedded)

`>>` shows up anywhere a wider hardware reading needs to be reduced to
a narrower range — a common example is a 12-bit ADC sample that needs
to become an 8-bit value, for something like an 8-bit PWM duty cycle:
shifting right by the difference in bit widths discards the low-order
bits cheaply, without the cost of a division. As in hosted code, it's
also the standard first step of extracting a specific field from a
packed register value before masking off the surrounding bits.

## Usage examples (Embedded)

### Downscaling a 12-bit ADC sample to an 8-bit duty cycle

```
fn adc_to_duty_cycle(sample_12bit: u16) -> u8 {
    (sample_12bit >> 4) as u8 // <- `>>` discards the low 4 bits, mapping 0..=4095 to 0..=255
}

assert_eq!(adc_to_duty_cycle(4095), 255);
assert_eq!(adc_to_duty_cycle(0), 0);
```
