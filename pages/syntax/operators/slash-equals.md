---
title: "/="
kind: operator
embedded_support: full
groups: [Arithmetic, Basics]
related_concepts: [Operator overloading]
related_syntax: ["/"]
see_also: ["/"]
---

## Explanation

`/=` divides the left operand by the right in place, overloadable via
`std::ops::DivAssign`.

## Basic usage example

```
let mut x = 7;
x /= 2; // <- `/=` divides `x` in place, truncating toward zero
```

## Best practices & deeper information

### Scenario: Numeric computation

Turning a running sum into a running average is a one-line job for
`/=` once the count is known — dividing the accumulator in place instead
of introducing a second variable to hold the averaged result.

```
let samples = [18.0, 22.0, 19.5, 24.5];

let mut average = samples.iter().sum::<f64>();
average /= samples.len() as f64; // <- `/=` turns the sum into an average in place

assert_eq!(average, 21.0);
```

**Why this way:** dividing the same binding in place with `/=` reads as
"this value, adjusted," matching the general in-place-assignment case
made for [`+=`](plus-equals.md); watch the integer-truncation caveat this
page's Explanation calls out if `average` were an integer type instead
of `f64`.

## Embedded Rust Notes

**Full support.** `DivAssign` lives in `core::ops` — same
software-division caveat as [`/`](slash.md) on dividerless targets.
