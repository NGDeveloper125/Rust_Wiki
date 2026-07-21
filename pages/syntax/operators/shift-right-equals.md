---
title: ">>="
kind: operator
embedded_support: full
groups: [Bitwise, Basics]
related_concepts: [Operator overloading]
related_syntax: [">>"]
see_also: [">>"]
---

## Explanation

`>>=` right-shifts the left operand by the right operand's amount, in
place, overloadable via `std::ops::ShrAssign`.

## Usage examples

### Right-shifting a value in place

```
let mut x = 8u8;
x >>= 3; // <- `>>=` right-shifts `x` in place
```

### Bit manipulation and flags

Unpacking a byte's fields one at a time by repeatedly consuming its
lowest bits is a natural use of `>>=` — each pass shifts the next field
into position while permanently discarding the bits already read.

```
let mut packed: u8 = 0b1011_0110; // 3 fields: 2 + 3 + 3 bits

let field_a = packed & 0b11;
packed >>= 2; // <- `>>=` discards the consumed 2 bits, shifts the rest down
let field_b = packed & 0b111;
packed >>= 3; // <- `>>=` consumes the next field the same way
let field_c = packed & 0b111;

assert_eq!((field_a, field_b, field_c), (0b10, 0b101, 0b101));
```

Mutating `packed` in place with `>>=` as each field is
consumed avoids tracking a separate shift-amount variable and re-deriving
it every time — the same in-place rationale as
[`+=`](plus-equals.md), here specialized to `ShrAssign`.

## Explanation (Embedded)

`>>=` is the mirror image of `<<=` for a bit-banged serial
*transmitter*: rather than assembling an incoming byte one sampled bit
at a time, the code starts with a complete byte and shifts it right
once per clock, reading off the current low bit (with `&`) to drive an
output pin before discarding it and bringing the next bit into
position.

## Usage examples (Embedded)

### Bit-banging a byte out over a software serial line, LSB first

```
fn set_output_pin(_high: bool) {
    // in real firmware this drives a GPIO pin high or low
}

fn bitbang_send_byte(mut data: u8) {
    for _ in 0..8 {
        set_output_pin(data & 1 != 0); // send the current low bit
        data >>= 1; // <- `>>=` discards the bit just sent, brings the next one into position
        // toggle_clock_pin();
    }
}
```
