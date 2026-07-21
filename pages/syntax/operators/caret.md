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

## Explanation (Embedded)

`^` carries over unchanged, and its self-cancelling property
(`x ^ y ^ y == x`) is exploited in two distinct embedded contexts. The
first is computing a value with some bits flipped relative to an
existing one *without* mutating anything — useful when a new register
value is being assembled ahead of a later write, as opposed to
toggling a register in place (that in-place case is `^=`, its own
page). The second is lightweight framing: many embedded serial
protocols use a single XOR byte as a cheap integrity check, since a
receiver can XOR every byte of an incoming frame together and compare
the result to a trailing checksum byte — catching single-bit
corruption on a wire without the code size or cycle cost of a CRC
table.

## Usage examples (Embedded)

### Computing a toggled register value without mutating the original

```
const LED_PIN: u32 = 1 << 5;

fn toggled_odr(current_odr: u32) -> u32 {
    current_odr ^ LED_PIN // <- `^` produces a new value with only the LED bit flipped
}

let odr = 0b0010_0000;
assert_eq!(toggled_odr(odr), 0);
```

### Checksumming a UART frame

```
fn frame_checksum(payload: &[u8]) -> u8 {
    payload.iter().fold(0u8, |acc, &b| acc ^ b) // <- `^` folds the frame into a single check byte
}

let frame = [0x02, 0xA4, 0x00, 0x7F]; // start byte, address, command, data
let checksum = frame_checksum(&frame);
```
