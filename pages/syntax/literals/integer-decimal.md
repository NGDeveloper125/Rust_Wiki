---
title: "Decimal integer literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["Numeric types & overflow behavior"]
related_syntax: [integer-suffixes, digit-separator]
see_also: [integer-hexadecimal, integer-octal, integer-binary]
---

## Explanation

The default, base-10 form for writing an integer literal:

```
let x = 42;
let y = 1_000_000;
```

With no suffix and no other context, Rust infers the type — defaulting to
`i32` if nothing constrains it further. Underscores (`_`) may be placed
anywhere between digits purely for readability; they carry no meaning and
don't affect the value (see [digit separator](digit-separator.md)). A
type suffix can be attached directly with no space (`42u8`, `1_000i64`)
to pin the literal's type explicitly — see
[integer suffixes](integer-suffixes.md).

## Embedded Rust Notes

**Full support.** Integer literals are core lexical grammar — identical
in `#![no_std]`. Embedded code leans heavily on explicit-width suffixes
(`u8`, `u16`, `u32`) since register widths and peripheral data sizes are
usually fixed and meaningful, unlike host code where `i32`/`usize`
defaults are often fine.
