---
title: "Binary integer literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["Numeric types & overflow behavior"]
related_syntax: [integer-suffixes, digit-separator]
see_also: [integer-decimal, integer-hexadecimal, integer-octal]
---

## Explanation

A base-2 integer literal, prefixed with `0b`, as in `0b1010_0101u8`.

Underscores are especially common here to group digits into readable
nibbles/bytes, as in that example — purely cosmetic, no effect on the
value.

## Usage examples

### Writing a binary bitmask

```
let mask = 0b0000_0001; // <- `0b` prefix marks a base-2 (binary) integer literal
```

Only the *digits* `0` and `1` may appear after the `0b`
prefix — any other digit is a compile error (though underscores and a
type suffix like `0b1010_u8` are still allowed).

### Bit manipulation and flags

A single status flag reads clearest written in binary — the bit position
is visible directly in the digits, with nothing to mentally translate.

```
const ENABLED: u8 = 0b0001; // <- binary literal: a single flag bit, position visible in the digits

let status: u8 = 0b0101;
if status & ENABLED != 0 {
    println!("device is enabled");
}
```

Writing a lone flag as `0b0001` keeps the bit position
obvious next to sibling flag constants; once a mask spans several bits at
once, hex reads more compactly — see
[integer-hexadecimal](integer-hexadecimal.md) for that convention, which
follows the same bitwise semantics documented on the
[std `BitAnd` trait](https://doc.rust-lang.org/std/ops/trait.BitAnd.html).

## Explanation (Embedded)

A binary literal means exactly the same thing under `#![no_std]` — pure
lexical grammar, resolved at compile time, with no allocator or runtime
involved. This is one of the two literal forms (alongside hex) that
carries genuine, everyday weight in firmware: register bit patterns and
masks are conventionally written in binary specifically when the
individual bit positions matter, since `0b1010_0000` shows every set bit
directly in the digits, with nothing to mentally translate the way a hex
or decimal equivalent would require. This reads best for single-byte
flag registers and small bitfields; once a mask spans a full 32-bit
register, the digit count gets unwieldy and hex (see
[integer-hexadecimal](integer-hexadecimal.md)) typically takes over —
but for the byte- and nibble-sized bitfields common in peripheral
control registers, binary is the clearer and more common choice.

## Usage examples (Embedded)

### Writing a GPIO pin's mode-register field bit by bit

```
const GPIO_MODER_PIN5_OUTPUT: u32 = 0b01 << 10; // <- binary literal: pin 5's 2-bit mode field, `01` = general-purpose output
```

### Building an interrupt-enable mask from individual bits

```
const TIMER_UPDATE_IE: u16 = 0b0000_0001;
const TIMER_CC1_IE: u16    = 0b0000_0010;
const TIMER_CC2_IE: u16    = 0b0000_0100;

let dier = TIMER_UPDATE_IE | TIMER_CC1_IE; // <- binary literal flags combined into one interrupt-enable register value
```
