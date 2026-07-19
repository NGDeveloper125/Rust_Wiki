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

A base-2 integer literal, prefixed with `0b`:

```
let flags = 0b1010_0101u8;
```

Underscores are especially common here to group digits into readable
nibbles/bytes, as shown above — purely cosmetic, no effect on the value.

## Basic usage example

```
let mask = 0b0000_0001; // <- `0b` prefix marks a base-2 (binary) integer literal
```

**Restriction:** only the digits `0` and `1` are legal after the `0b`
prefix — anything else is a compile error.

## Best practices & deeper information

### Scenario: Bit manipulation and flags

A single status flag reads clearest written in binary — the bit position
is visible directly in the digits, with nothing to mentally translate.

```
const ENABLED: u8 = 0b0001; // <- binary literal: a single flag bit, position visible in the digits

let status: u8 = 0b0101;
if status & ENABLED != 0 {
    println!("device is enabled");
}
```

**Why this way:** writing a lone flag as `0b0001` keeps the bit position
obvious next to sibling flag constants; once a mask spans several bits at
once, hex reads more compactly — see
[integer-hexadecimal](integer-hexadecimal.md) for that convention, which
follows the same bitwise semantics documented on the
[std `BitAnd` trait](https://doc.rust-lang.org/std/ops/trait.BitAnd.html).

## Embedded Rust Notes

**Full support.** Binary literals are extremely common in embedded code
for expressing register bit patterns and masks directly
(`0b0000_0001`) — no `std` dependency.
