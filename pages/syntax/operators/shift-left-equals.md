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

## Basic usage example

```
let mut x = 1u8;
x <<= 3; // <- `<<=` left-shifts `x` in place
```

## Best practices & deeper information

### Scenario: Bit manipulation and flags

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

**Why this way:** accumulating into one `mut` binding with `<<=` avoids
building and discarding an intermediate value on every iteration — the
same in-place-mutation case made for [`+=`](plus-equals.md), here
specialized to `ShlAssign`.

## Embedded Rust Notes

**Full support.** `ShlAssign` lives in `core::ops` — no `std` dependency.
