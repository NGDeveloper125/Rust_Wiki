---
title: "&="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["&"]
see_also: ["&"]
---

## Explanation

`&=` assigns the bitwise AND of the left and right operands in place, as
in `flags &= 0b1010`, and is overloadable via `std::ops::BitAndAssign`.
Only the integer/bitwise sense of `&` has a compound-assignment form —
there's no analogous compound form for the borrow sense of `&` (borrowing
isn't a "value" that can be compounded like this).

## Basic usage example

```
let mut flags = 0b1100u8;
flags &= 0b1010; // <- clears bits not set in the mask, in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `&=` assigns in place.

## Best practices & deeper information

### Scenario: Bit manipulation and flags

Clearing a single flag bit without disturbing the rest of the set is the
classic use of `&=`: AND the current bits against the complement of the
bit you want gone.

```
const FLAG_ACTIVE: u8   = 0b0001;
const FLAG_PENDING: u8  = 0b0010;
const FLAG_ARCHIVED: u8 = 0b0100;

let mut status: u8 = FLAG_ACTIVE | FLAG_PENDING | FLAG_ARCHIVED;
status &= !FLAG_PENDING; // <- clears the PENDING bit in place, keeping the others
assert_eq!(status, FLAG_ACTIVE | FLAG_ARCHIVED);
```

**Why this way:** `flags &= !bit` is the standard bit-clearing idiom in
systems code generally, built on the
[`BitAndAssign`](https://doc.rust-lang.org/std/ops/trait.BitAndAssign.html)
trait behind `&=` — see [`+=`](plus-equals.md) for the fuller treatment
of compound-assignment operators shared across `+=`, `-=`, `*=`, and the
rest of the family.

## Embedded Rust Notes

**Full support.** `BitAndAssign` lives in `core::ops` — clearing specific
bits in a register (`reg &= !mask`) is routine embedded code.
