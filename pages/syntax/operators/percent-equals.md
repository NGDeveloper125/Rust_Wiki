---
title: "%="
kind: operator
embedded_support: full
groups: [Arithmetic, Basics]
related_concepts: [Operator overloading]
related_syntax: ["%"]
see_also: ["%"]
---

## Explanation

`%=` assigns the remainder of the left operand divided by the right,
overloadable via `std::ops::RemAssign`.

## Usage examples

### Assigning a remainder back into a variable

```
let mut x = 7;
x %= 2; // <- `%=` assigns the remainder of `x / 2` back into `x`
```

### Numeric computation

A ring buffer's write index needs to wrap back to `0` once it reaches the
buffer's length — `%=` expresses "wrap this index in place" in one step,
without a separate comparison-and-reset branch.

```
struct RingBuffer {
    data: [u8; 8],
    idx: usize,
}

impl RingBuffer {
    fn push(&mut self, byte: u8) {
        self.data[self.idx] = byte;
        self.idx += 1;
        self.idx %= self.data.len(); // <- `%=` wraps `idx` back into range in place
    }
}

let mut buf = RingBuffer { data: [0; 8], idx: 7 };
buf.push(42);
assert_eq!(buf.idx, 0); // wrapped from 7 back to 0
```

`idx %= len` is the idiomatic circular-index pattern —
see [`+=`](plus-equals.md) for the general in-place-assignment rationale;
the same "mutate the field directly" logic applies here, and the modulo
avoids an explicit `if idx == len { idx = 0 }` branch.

## Embedded Rust Notes

**Full support.** `RemAssign` lives in `core::ops` — no `std` dependency.
