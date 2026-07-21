---
title: "^"
kind: operator
embedded_support: full
groups: [Bitwise, Basics]
related_concepts: [Operator overloading]
related_syntax: ["^="]
see_also: ["^="]
---

## Explanation

`^` is bitwise exclusive OR (XOR) between integers, overloadable via
`std::ops::BitXor`.

Also commonly used with `bool` as an XOR/"exactly one of" operator, since
`BitXor` is implemented for `bool` as well as the integer types (unlike
`&&`/`||`, `^` never short-circuits — both operands are always evaluated).

## Usage examples

### Computing a bitwise XOR

```
let x = 0b1100 ^ 0b1010; // <- bitwise XOR: bits that differ become 1
```

### Bit manipulation and flags

XOR-ing a byte into a running accumulator, then XOR-ing the same byte in
again later, cancels out — the property that makes `^` a cheap building
block for lightweight checksums.

```
fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |acc, &b| acc ^ b) // <- `^` folds each byte into a running XOR
}

let packet = [0x3A, 0x7F, 0x01, 0xEE];
let check = checksum(&packet);
println!("checksum: {check:#04x}");
```

XOR is mathematically self-cancelling (`x ^ y ^ y ==
x`), and that property is exactly why `^` (rather than `&`/`|`) — the
operator behind the [`BitXor`](https://doc.rust-lang.org/std/ops/trait.BitXor.html)
trait — is the natural choice whenever "combine, and let duplicates
cancel" is the goal, as with simple checksums or toggle masks.

## Embedded Rust Notes

**Full support.** `BitXor` lives in `core::ops` — bit-toggling a hardware
register (`reg ^= mask`) is common in embedded code, e.g. toggling an
output pin.
