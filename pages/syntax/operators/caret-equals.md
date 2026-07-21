---
title: "^="
kind: operator
embedded_support: full
groups: [Bitwise, Basics]
related_concepts: [Operator overloading]
related_syntax: ["^"]
see_also: ["^"]
---

## Explanation

`^=` assigns the bitwise XOR of the left and right operands in place,
overloadable via `std::ops::BitXorAssign`. A classic use is toggling bits:
`flags ^= mask` flips exactly the bits set in `mask`.

## Usage examples

### Toggling bits in place

```
let mut flags = 0b1010u8;
flags ^= 0b0011; // <- toggles the bits set in the mask, in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `^=` assigns in place.

### Bit manipulation and flags

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

Toggling with `^=` is the documented idiom on
[`BitXorAssign`](https://doc.rust-lang.org/std/ops/trait.BitXorAssign.html)
for flipping a bit regardless of its current value — see
[`+=`](plus-equals.md) for the general notes shared across the
compound-assignment operator family.

## Explanation (Embedded)

`^=` is the standard way to flip a hardware output bit without needing
to know its current state first: many microcontroller GPIO ports have
no dedicated "toggle" register (only the plain output-data register),
so firmware reads the port's current value, XORs in the pin's bit, and
writes the result back. Because XOR-ing the same bit twice restores the
original value, `reg ^= mask` is self-inverse — calling the same toggle
function twice in a row leaves the pin exactly where it started, which
is exactly the behavior wanted for something like blinking an LED once
per timer tick.

## Usage examples (Embedded)

### Toggling an LED pin via a read-modify-write

```
const GPIOA_ODR: *mut u32 = 0x4001_080C as *mut u32; // GPIOA output data register
const LED_PIN: u32 = 1 << 5;

fn toggle_led() {
    unsafe {
        let mut odr = core::ptr::read_volatile(GPIOA_ODR);
        odr ^= LED_PIN; // <- flips only the LED bit, in place
        core::ptr::write_volatile(GPIOA_ODR, odr);
    }
}
```
