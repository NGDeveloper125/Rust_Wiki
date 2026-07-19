---
title: "^="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["^"]
see_also: ["^"]
---

## Explanation

`^=` assigns the bitwise XOR of the left and right operands in place,
overloadable via `std::ops::BitXorAssign`. A classic use is toggling bits:
`flags ^= mask` flips exactly the bits set in `mask`.

```
let mut flags = 0b1010u8;
flags ^= 0b0110; // flags is now 0b1100
```

## Basic usage example

```
let mut flags = 0b1010u8;
flags ^= 0b0011; // <- toggles the bits set in the mask, in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `^=` assigns in place.

## Best practices & deeper information

### Scenario: Bit manipulation and flags

`^=` flips a specific bit without needing to know its current state
first — XOR-ing a bit in twice restores the original value.

```
const FLAG_NOTIFY: u8 = 0b0001;
const FLAG_MUTED: u8  = 0b0010;

let mut settings: u8 = FLAG_NOTIFY;
settings ^= FLAG_MUTED; // <- toggles the MUTED bit on, leaving NOTIFY untouched
settings ^= FLAG_MUTED; // toggling the same bit again restores the original value
assert_eq!(settings, FLAG_NOTIFY);
```

**Why this way:** toggling with `^=` is the documented idiom on
[`BitXorAssign`](https://doc.rust-lang.org/std/ops/trait.BitXorAssign.html)
for flipping a bit regardless of its current value — see
[`+=`](plus-equals.md) for the general notes shared across the
compound-assignment operator family.

## Embedded Rust Notes

**Full support.** `BitXorAssign` lives in `core::ops` — no `std`
dependency.
