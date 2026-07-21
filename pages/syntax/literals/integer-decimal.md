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

The default, base-10 form for writing an integer literal, as in `42` or
`1_000_000`.

With no suffix and no other context, Rust infers the type — defaulting to
`i32` if nothing constrains it further. Underscores (`_`) may be placed
anywhere between digits purely for readability; they carry no meaning and
don't affect the value (see [digit separator](digit-separator.md)). A
type suffix can be attached directly with no space (`42u8`, `1_000i64`)
to pin the literal's type explicitly — see
[integer suffixes](integer-suffixes.md).

## Usage examples

### Writing everyday quantities in decimal

```
let count = 42; // <- decimal integer literal: base 10, no prefix needed
```

### Numeric computation

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

`checked_mul` turns a would-be silent overflow into an
explicit `None` the caller must handle — the
[std docs for `checked_mul`](https://doc.rust-lang.org/std/primitive.u32.html#method.checked_mul)
document that `None`-on-overflow behavior. Preferring it over plain `*`
whenever an operand could come from outside the function avoids that
silent overflow.

### Creating a new object

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

Implementing `Default` with straightforward literal
values documents the intended starting point at one place, which the
[API Guidelines' C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits)
recommends for any type with an obvious default.

## Explanation (Embedded)

A decimal literal is core lexical grammar, identical under `#![no_std]`
— honestly, this is the least embedded-specific of the integer literal
forms. Addresses and bitmasks gravitate to hex or binary (see
[integer-hexadecimal](integer-hexadecimal.md) and
[integer-binary](integer-binary.md)) precisely because those bases show
byte/bit structure that decimal hides, so decimal's role in firmware is
the same plain one it has anywhere else: everyday counts, loop bounds,
buffer sizes, delays, and retry limits, with nothing about the base
itself changing under `no_std`. (See [integer suffixes](integer-suffixes.md)
for where embedded code *does* diverge from typical host code —
explicit-width type suffixes, not the choice of decimal versus another
base.)

## Usage examples (Embedded)

### Sizing a heapless buffer with a plain decimal constant

```
use heapless::Vec;

const SAMPLE_COUNT: usize = 32; // <- decimal literal: a buffer capacity, nothing embedded-specific about the base
let mut samples: Vec<u16, SAMPLE_COUNT> = Vec::new();
```

### Retrying a sensor read a fixed number of times

```
fn read_with_retries<E>(mut read: impl FnMut() -> Result<u16, E>) -> Result<u16, E> {
    let max_attempts = 3; // <- decimal literal: a plain retry count, same as it would be in hosted code
    let mut last_err = None;
    for _ in 0..max_attempts {
        match read() {
            Ok(v) => return Ok(v),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap())
}
```
