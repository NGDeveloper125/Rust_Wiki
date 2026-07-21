---
title: "|="
kind: operator
embedded_support: full
groups: [Bitwise, Basics]
related_concepts: [Operator overloading]
related_syntax: ["|"]
see_also: ["|"]
---

## Explanation

`|=` assigns the bitwise OR of the left and right operands in place,
overloadable via `std::ops::BitOrAssign`. Commonly used to set flag bits
in a bitmask.

## Usage examples

### ORing a value into a variable in place

```
let mut flags = 0b1000u8;
flags |= 0b0010; // <- `|=` ORs the right operand into `flags` in place
```

### Bit manipulation and flags

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

`status |= FLAG` mutates the existing word in place
instead of rebuilding it from scratch, which matters once other bits are
already meaningfully set — the same in-place-mutation reasoning as
[`+=`](plus-equals.md), specialized to bitwise OR via `BitOrAssign`.

## Explanation (Embedded)

`|=` is the read-modify-write idiom for setting one or more bits in a
register while leaving the rest alone, and it's arguably the single
most common register operation in embedded Rust: enabling a
peripheral's clock in a clock-control register, setting a
configuration bit in a control register, or marking one flag in a
software status word are all `reg |= BIT`. Some microcontrollers
additionally expose a dedicated atomic "set" register (STM32's GPIO
`BSRR`, for instance) specifically so a single bit can be set without a
read-modify-write race against an interrupt handler touching the same
port; where a peripheral doesn't offer that, `|=` on the plain data
register is the fallback, and is fine as long as nothing else can
preempt the read and the write.

## Usage examples (Embedded)

### Enabling a peripheral clock before configuring it

```
const RCC_APB1ENR: *mut u32 = 0x4002_1840 as *mut u32; // RCC APB1 clock-enable register
const USART2EN: u32 = 1 << 17;

fn enable_usart2_clock() {
    unsafe {
        let mut enr = core::ptr::read_volatile(RCC_APB1ENR);
        enr |= USART2EN; // <- sets only the USART2 clock-enable bit
        core::ptr::write_volatile(RCC_APB1ENR, enr);
    }
}
```
