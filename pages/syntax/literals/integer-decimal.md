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

## Basic usage example

```
let count = 42; // <- decimal integer literal: base 10, no prefix needed
```

## Best practices & deeper information

### Scenario: Numeric computation

Decimal is the natural base for everyday quantities — order counts, prices
in cents — and feeds straight into checked arithmetic when the inputs
might overflow.

```
fn total_cents(unit_price_cents: u32, quantity: u32) -> Option<u32> {
    unit_price_cents.checked_mul(quantity)
}

let subtotal = total_cents(1999, 3); // <- decimal literals: the natural base for prices and counts
assert_eq!(subtotal, Some(5997));
```

**Why this way:** `checked_mul` turns a would-be silent overflow into an
explicit `None` the caller must handle — the
[std docs for `checked_mul`](https://doc.rust-lang.org/std/primitive.u32.html#method.checked_mul)
document that `None`-on-overflow behavior. Preferring it over plain `*`
whenever an operand could come from outside the function is the safe
default.

### Scenario: Creating a new object

A `Default` impl built from plain decimal literals gives callers a
documented, zero-argument starting point instead of forcing every call
site to repeat the same constants.

```
struct RetryPolicy {
    max_attempts: u32,
    backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_attempts: 3,   // <- decimal literal: a sensible default value
            backoff_ms: 500,   // <- decimal literal: a sensible default value
        }
    }
}
```

**Why this way:** implementing `Default` with straightforward literal
values documents the intended starting point at one place, which the
[API Guidelines' C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits)
recommends for any type with an obvious default.

## Embedded Rust Notes

**Full support.** Integer literals are core lexical grammar — identical
in `#![no_std]`. Embedded code leans heavily on explicit-width suffixes
(`u8`, `u16`, `u32`) since register widths and peripheral data sizes are
usually fixed and meaningful, unlike host code where `i32`/`usize`
defaults are often fine.
