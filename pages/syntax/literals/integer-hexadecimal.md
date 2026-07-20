---
title: "Hexadecimal integer literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["Numeric types & overflow behavior"]
related_syntax: [integer-suffixes, digit-separator]
see_also: [integer-decimal, integer-octal, integer-binary]
---

## Explanation

A base-16 integer literal, prefixed with `0x`:

```
let mask = 0xFF_u8;
let color = 0x1a2b3c;
```

Digits `a`–`f` may be upper- or lower-case, and can be mixed within the
same literal (though consistent casing is the usual style convention).
Underscores are allowed between digits, including immediately after the
`0x` prefix. Like all integer literals, an optional type suffix
(`0xffu8`) can pin the type directly.

## Basic usage example

```
let addr = 0x2000; // <- `0x` prefix marks a base-16 (hexadecimal) integer literal
```

**Restriction:** only the hex *digits* `0`–`9` and `a`–`f`/`A`–`F` may
appear after the `0x` prefix (underscores and a type suffix like
`0xFF_u8` are still allowed).

## Best practices & deeper information

### Scenario: Bit manipulation and flags

Hex groups bits into 4-bit nibbles, so a multi-bit register mask reads as
"which nibble, which bits" at a glance — far harder to see in decimal.

```
const TX_ENABLE: u32   = 0x0000_0001;
const RX_ENABLE: u32   = 0x0000_0002;
const PARITY_EVEN: u32 = 0x0000_0004;
const BAUD_MASK: u32   = 0x0000_0F00; // <- hex literal: a multi-bit mask, nibble boundaries stay visible

fn configure(control: u32) -> u32 {
    control | TX_ENABLE | RX_ENABLE | PARITY_EVEN
}

let baud_bits = configure(0) & BAUD_MASK;
```

**Why this way:** hex digits map exactly onto 4-bit groups, so a mask like
`0x0F00` communicates "bits 8–11" directly — the readability reason
register-level code conventionally writes masks in hex rather than
decimal.

### Scenario: Numeric computation

Power-of-two sizes and addresses read as recognizable patterns in hex in
a way their decimal equivalents don't, which matters for anything doing
alignment arithmetic.

```
const PAGE_SIZE: usize = 0x1000; // <- hex literal: a power-of-two boundary reads clearly in hex

fn is_page_aligned(addr: usize) -> bool {
    addr & (PAGE_SIZE - 1) == 0
}

assert!(is_page_aligned(0x4000));
assert!(!is_page_aligned(0x4010));
```

**Why this way:** `0x1000` and `0x4000` are instantly recognizable as
round, aligned values in a way `4096` and `16384` aren't — which is why
systems code conventionally writes addresses and alignment constants in
hex.

## Embedded Rust Notes

**Full support.** Hex literals are the conventional way to write register
addresses and bitmasks in embedded code (`0x4001_0000`, `0xFF`) — no
`std` dependency.
