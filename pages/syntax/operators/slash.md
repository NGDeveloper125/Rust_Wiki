---
title: "/"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["/="]
see_also: ["/="]
---

## Explanation

`/` is arithmetic division, overloadable via `std::ops::Div`:

```
let quotient = 7 / 2; // 3 for integers — truncates toward zero
```

Integer division truncates rather than rounding; `7 / 2 == 3` and
`-7 / 2 == -3`. Dividing an integer by zero panics unconditionally (even
in release builds) rather than producing infinity or undefined behavior;
floating-point division by zero instead follows IEEE 754 and produces
`inf`, `-inf`, or `NaN`.

## Basic usage example

```
let quotient = 7 / 2; // <- `/` divides, truncating toward zero
```

**Restriction:** dividing an integer by zero panics unconditionally, even
in release builds.

## Best practices & deeper information

### Scenario: Numeric computation

Computing an average from a sum and a count is ordinary `/`, and the
truncation behavior this page's Explanation describes is exactly why
integer division needs a moment's thought before it's used for anything
that isn't meant to be a whole number.

```
let total_ms: u32 = 1_205;
let sample_count: u32 = 4;

let average_ms = total_ms / sample_count; // <- `/` truncates: 301, not 301.25
assert_eq!(average_ms, 301);

let average_ms_f = total_ms as f64 / sample_count as f64; // 301.25, precise
```

**Why this way:** casting to a float before dividing is the standard fix
when truncation would silently discard a meaningful fraction — the
[standard library docs on integer types](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_div)
are explicit that integer `/` always rounds toward zero, never to the
nearest value.

### Scenario: Handling and propagating errors

Where the divisor isn't guaranteed to be nonzero — say, an average
computed over a possibly-empty batch of readings — `checked_div` turns
the panic this page warns about into an `Option` the caller must handle.

```
fn average(total: i32, count: i32) -> Option<i32> {
    total.checked_div(count) // returns None instead of panicking when count == 0
}

assert_eq!(average(100, 4), Some(25));
assert_eq!(average(100, 0), None); // no panic: an empty batch is a normal case
```

**Why this way:** `checked_div` makes the zero-divisor case an explicit
branch instead of an unconditional panic, which the
[standard library docs](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_div)
recommend whenever the divisor comes from data the function doesn't
fully control (as opposed to a compile-time-known nonzero constant,
where bare `/` is fine).

## Embedded Rust Notes

**Full support.** `Div` lives in `core::ops`. Worth knowing: many small
microcontrollers have no hardware integer divider, so `/` on those
targets compiles to a (slower) software division routine — profile before
assuming it's free.
