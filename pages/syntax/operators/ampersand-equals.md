---
title: "&="
kind: operator
embedded_support: full
groups: [Bitwise, Basics]
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

## Usage examples

### Clearing bits with a mask

```
let mut flags = 0b1100u8;
flags &= 0b1010; // <- clears bits not set in the mask, in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `&=` assigns in place.

### Bit manipulation and flags

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

`flags &= !bit` is the standard bit-clearing idiom in
systems code generally, built on the
[`BitAndAssign`](https://doc.rust-lang.org/std/ops/trait.BitAndAssign.html)
trait behind `&=` — see [`+=`](plus-equals.md) for the fuller treatment
of compound-assignment operators shared across `+=`, `-=`, `*=`, and the
rest of the family.

## Explanation (Embedded)

`&=` is the read-modify-write idiom for clearing one or more bits in a
register while leaving every other bit alone: AND the current value
against the bitwise complement of the bits to clear. This is one of
the most routine operations in embedded firmware — disabling a single
interrupt source in an enable register, or clearing one configuration
bit in a peripheral's control register — anywhere the hardware doesn't
offer a dedicated "clear" register and software has to preserve the
bits it isn't touching by hand.

## Usage examples (Embedded)

### Disabling one interrupt source in an interrupt-enable register

```
const TIMER_IER: *mut u32 = 0x4000_0C00 as *mut u32; // timer interrupt-enable register
const UPDATE_IE: u32 = 1 << 0; // update-event interrupt enable bit

fn disable_update_interrupt() {
    unsafe {
        let mut ier = core::ptr::read_volatile(TIMER_IER);
        ier &= !UPDATE_IE; // <- clears only the update-interrupt bit, leaves the rest as they were
        core::ptr::write_volatile(TIMER_IER, ier);
    }
}
```
