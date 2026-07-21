---
title: "<<="
kind: operator
embedded_support: full
groups: [Bitwise, Basics]
related_concepts: [Operator overloading]
related_syntax: ["<<"]
see_also: ["<<"]
---

## Explanation

`<<=` left-shifts the left operand by the right operand's amount, in
place, overloadable via `std::ops::ShlAssign`.

## Usage examples

### Left-shifting a value in place

```
let mut x = 1u8;
x <<= 3; // <- `<<=` left-shifts `x` in place
```

### Bit manipulation and flags

Building up a bitmask one field at a time — shifting previously-packed
bits further left to make room for the next field — is a natural fit for
`<<=`, mutating the accumulator in place across a loop.

```
let field_widths = [3u8, 2, 4]; // bits per field, packed low to high

let mut mask = 0u16;
for &width in &field_widths {
    mask <<= width; // <- `<<=` shifts the accumulated mask left to make room
    mask |= (1 << width) - 1; // fill the freed low bits with 1s for this field
}

assert_eq!(mask, 0b0001_1111_1111);
```

Accumulating into one `mut` binding with `<<=` avoids
building and discarding an intermediate value on every iteration — the
same in-place-mutation case made for [`+=`](plus-equals.md), here
specialized to `ShlAssign`.

## Explanation (Embedded)

`<<=` shows up in embedded code anywhere a byte (or word) is being
assembled one bit at a time from a hardware source that only exposes a
single bit per sample. The classic case is bit-banging a serial
protocol in software rather than through a dedicated peripheral: on
each clock edge, the code samples a data pin, shifts the value built so
far one place to the left to make room, and ORs the new bit into the
now-empty low position.

## Usage examples (Embedded)

### Bit-banging a byte in over a software SPI line

```
fn read_bit_from_miso() -> u32 {
    // in real firmware this samples the MISO GPIO pin
    1
}

fn bitbang_receive_byte() -> u8 {
    let mut value: u8 = 0;
    for _ in 0..8 {
        value <<= 1; // <- `<<=` makes room for the next incoming bit, MSB first
        value |= read_bit_from_miso() as u8;
        // toggle_clock_pin();
    }
    value
}
```
