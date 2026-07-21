---
title: "%"
kind: operator
embedded_support: full
groups: [Arithmetic, Basics]
related_concepts: [Operator overloading]
related_syntax: ["%="]
see_also: ["%="]
---

## Explanation

`%` is the remainder operator, overloadable via `std::ops::Rem`.

It's the *remainder*, not strictly modulo — for negative operands, the
result takes the sign of the dividend (`-7 % 2 == -1`, not `1`), which
differs from the mathematical modulo used by some other languages. Like
`/`, `%` panics on division by zero for integers.

## Usage examples

### Computing a remainder

```
let r = 7 % 2; // <- `%` computes the remainder
```

**Restriction:** dividing (or taking the remainder) by zero panics
unconditionally for integers, even in release builds.

### Numeric computation

Wrapping a ring buffer index into range is one of `%`'s most common
jobs — `(idx + 1) % len` computes the next slot without ever needing a
branch to reset back to zero.

```
struct RingBuffer {
    slots: [u8; 4],
    next: usize,
}

impl RingBuffer {
    fn advance(&mut self) {
        self.next = (self.next + 1) % self.slots.len(); // <- `%` wraps the index
    }
}

let mut buf = RingBuffer { slots: [0; 4], next: 3 };
buf.advance();
assert_eq!(buf.next, 0); // wrapped past the end back to slot 0
```

`% len` only wraps correctly because `len` is never
zero for a fixed-size array — for a length that could legitimately be
zero, prefer `checked_rem` over risking the panic this page's
Explanation calls out, per the
[standard library docs](https://doc.rust-lang.org/std/primitive.usize.html#method.checked_rem).

## Explanation (Embedded)

`Rem` lives in `core::ops`, so `%` means exactly the same thing under
`#![no_std]`, including the sign-of-the-dividend behavior and the
unconditional panic on a zero divisor. It shares [`/`](slash.md)'s
hardware caveat: on a microcontroller with no integer divider, `%`
lowers to the same software division routine, since remainder and
quotient are typically computed by the same instruction/routine
underneath. The genuinely embedded-flavored idiom that follows from this
is sizing a firmware ring buffer or FIFO to a power of two on purpose —
common enough that it's practically a convention — specifically so
wrapping an index can use a bitmask (`idx & (len - 1)`) instead of `%`,
skipping the division routine entirely. `%` itself stays the right,
clearer choice whenever the length isn't a power of two or the saved
cycles don't matter.

## Usage examples (Embedded)

### Wrapping a DMA buffer index with `%`

```
struct DmaBuffer {
    slots: [u16; 6],
    next: usize,
}

impl DmaBuffer {
    fn advance(&mut self) {
        self.next = (self.next + 1) % self.slots.len(); // <- `%` wraps the index, same as hosted code
    }
}
```

### Replacing `%` with a bitmask on a power-of-two buffer

```
struct RingBuffer {
    data: [u8; 16], // length is a power of two: 16
    idx: usize,
}

impl RingBuffer {
    fn push(&mut self, byte: u8) {
        self.data[self.idx] = byte;
        self.idx = (self.idx + 1) & (self.data.len() - 1); // mask replaces `%`: no division instruction needed
    }
}
```
