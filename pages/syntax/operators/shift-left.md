---
title: "<<"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["<<=", ">>"]
see_also: [">>"]
---

## Explanation

`<<` is the left-shift operator, overloadable via `std::ops::Shl`:

```
let x = 1u8 << 3; // 8
```

Shifting by an amount greater than or equal to the type's bit width is a
panic in debug builds (and unspecified/masked behavior to guard against in
release — check `checked_shl`/`wrapping_shl` for defined behavior at the
boundary). `<<` on integers is a pure bit-shift, unrelated to C++'s
overload of `<<` for stream output — Rust uses `{}`/`write!` and the
`Display`/`Debug` traits for formatting instead.

## Basic usage example

```
let x = 1u8 << 3; // <- `<<` shifts the bits of `1u8` left by 3
```

**Restriction:** shifting by an amount greater than or equal to the
type's bit width panics in debug builds; use `checked_shl`/`wrapping_shl`
for defined behavior at the boundary.

## Best practices & deeper information

### Scenario: Bit manipulation and flags

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

**Why this way:** `1 << pin` names the intent ("the bit at this
position") directly, whereas hand-computing the equivalent power-of-two
literal invites off-by-one mistakes; the
[Rust by Example bitwise chapter](https://doc.rust-lang.org/rust-by-example/primitives/literals.html)
shows the same shift-to-build-a-mask idiom.

## Embedded Rust Notes

**Full support.** `Shl` lives in `core::ops` — bit shifts are used
constantly in embedded code to construct register masks (`1 << pin_num`).
