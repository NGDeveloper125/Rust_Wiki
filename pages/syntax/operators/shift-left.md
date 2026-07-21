---
title: "<<"
kind: operator
embedded_support: full
groups: [Bitwise, Basics]
related_concepts: [Operator overloading]
related_syntax: ["<<=", ">>"]
see_also: [">>"]
---

## Explanation

`<<` is the left-shift operator, overloadable via `std::ops::Shl`.

Shifting by an amount greater than or equal to the type's bit width
panics in debug builds; when overflow checks are off (release builds),
Rust guarantees the shift amount is masked to the type's bit width —
defined behavior, never UB, though rarely what you meant (use
`checked_shl`/`wrapping_shl` to make the boundary case explicit). `<<` on
integers is a pure bit-shift, unrelated to C++'s
overload of `<<` for stream output — Rust uses `{}`/`write!` and the
`Display`/`Debug` traits for formatting instead.

## Usage examples

### Left-shifting the bits of an integer

```
let x = 1u8 << 3; // <- `<<` shifts the bits of `1u8` left by 3
```

**Restriction:** shifting by an amount greater than or equal to the
type's bit width panics in debug builds; with overflow checks off, the
shift amount is masked to the bit width (defined, but usually not what
you meant) — use `checked_shl`/`wrapping_shl` to make the boundary case
explicit.

### Bit manipulation and flags

Turning a bit position into a single-bit mask — "the bit for pin 5" — is
one of `<<`'s most common jobs, and reads far more clearly than writing
out the equivalent literal by hand.

```
const PIN_COUNT: u8 = 8;

fn pin_mask(pin: u8) -> u8 {
    debug_assert!(pin < PIN_COUNT);
    1 << pin // <- `<<` turns a pin index into a single set bit
}

let enabled = pin_mask(0) | pin_mask(3) | pin_mask(5);
assert_eq!(enabled, 0b0010_1001);
```

`1 << pin` names the intent ("the bit at this
position") directly, whereas hand-computing the equivalent power-of-two
literal invites off-by-one mistakes; the
[Rust by Example bitwise chapter](https://doc.rust-lang.org/rust-by-example/primitives/literals.html)
shows the same shift-to-build-a-mask idiom.

## Explanation (Embedded)

`<<` carries over unchanged and is the standard way to position a
value at a specific bit offset within a register, rather than at bit
0. Many peripheral registers pack several unrelated fields into one
word — a UART's word length, parity, and stop-bit count might each
occupy a couple of bits at different offsets — and writing one of
those fields means shifting its value up to the field's offset before
OR-ing it into the register (see [`|`](pipe.md)). That's distinct from
the single-bit case of turning a pin index into a mask (`1 << pin`):
the same operator, but shifting a value *wider than one bit* into
position rather than a single set bit.

## Usage examples (Embedded)

### Positioning a multi-bit field within a control register

```
const STOP_BITS_OFFSET: u32 = 12; // USART CR2 STOP field starts at bit 12
const STOP_BITS_2: u32 = 0b10;    // "2 stop bits", per the field's own encoding

fn cr2_value() -> u32 {
    STOP_BITS_2 << STOP_BITS_OFFSET // <- `<<` moves the 2-bit field up to its position in the register
}

assert_eq!(cr2_value(), 0b10_0000_0000_0000);
```
