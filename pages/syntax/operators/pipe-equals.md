---
title: "|="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["|"]
see_also: ["|"]
---

## Explanation

`|=` assigns the bitwise OR of the left and right operands in place,
overloadable via `std::ops::BitOrAssign`. Commonly used to set flag bits
in a bitmask.

## Basic usage example

```
let mut flags = 0b1000u8;
flags |= 0b0010; // <- `|=` ORs the right operand into `flags` in place
```

## Best practices & deeper information

### Scenario: Bit manipulation and flags

Setting an individual flag bit on a status word is the canonical `|=`
use — it turns one bit on without disturbing the others, in place.

```
const FLAG_READY: u8   = 0b0000_0001;
const FLAG_ERROR: u8   = 0b0000_0010;
const FLAG_LOGGING: u8 = 0b0000_0100;

let mut status = 0u8;
status |= FLAG_READY;   // <- `|=` sets the READY bit, leaves others untouched
status |= FLAG_LOGGING; // <- `|=` sets LOGGING on top of READY

assert_eq!(status, FLAG_READY | FLAG_LOGGING);
assert_eq!(status & FLAG_ERROR, 0); // ERROR was never touched
```

**Why this way:** `status |= FLAG` mutates the existing word in place
instead of rebuilding it from scratch, which matters once other bits are
already meaningfully set — the same in-place-mutation reasoning as
[`+=`](plus-equals.md), specialized to bitwise OR via `BitOrAssign`.

## Embedded Rust Notes

**Full support.** `BitOrAssign` lives in `core::ops` — setting flag bits
in place (`reg |= mask`) is routine embedded register manipulation.
