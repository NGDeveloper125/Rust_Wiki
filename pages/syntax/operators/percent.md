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

## Basic usage example

```
let r = 7 % 2; // <- `%` computes the remainder
```

**Restriction:** dividing (or taking the remainder) by zero panics
unconditionally for integers, even in release builds.

## Best practices & deeper information

### Scenario: Numeric computation

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

**Why this way:** `% len` only wraps correctly because `len` is never
zero for a fixed-size array — for a length that could legitimately be
zero, prefer `checked_rem` over risking the panic this page's
Explanation calls out, per the
[standard library docs](https://doc.rust-lang.org/std/primitive.usize.html#method.checked_rem).

## Embedded Rust Notes

**Full support.** `Rem` lives in `core::ops` — same software-division
caveat as [`/`](slash.md) on dividerless targets.
