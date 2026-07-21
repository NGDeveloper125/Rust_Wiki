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

## Usage examples

### Dividing a mutable binding in place

```
let mut x = 7;
x /= 2; // <- `/=` divides `x` in place, truncating toward zero
```

### Numeric computation

Turning a running sum into a running average is a one-line job for
`/=` once the count is known — dividing the accumulator in place instead
of introducing a second variable to hold the averaged result.

```
let samples = [18.0, 22.0, 19.5, 24.5];

let mut average = samples.iter().sum::<f64>();
average /= samples.len() as f64; // <- `/=` turns the sum into an average in place

assert_eq!(average, 21.0);
```

Dividing the same binding in place with `/=` reads as
"this value, adjusted," matching the general in-place-assignment case
made for [`+=`](plus-equals.md); watch the integer-truncation caveat this
page's Explanation calls out if `average` were an integer type instead
of `f64`.

## Explanation (Embedded)

`DivAssign` lives in `core::ops`, so `/=` behaves identically under
`#![no_std]`, including the integer-truncation caveat this page's classic
Explanation notes. It shares [`/`](slash.md)'s hardware nuance directly:
on a microcontroller with no hardware integer divider, `/=` lowers to the
same software division routine `/` does. When the divisor is a
compile-time-known power of two — a common choice for a sample count or
buffer length precisely because of this — the compiler turns `/=` into a
shift for you, making it effectively free; a divisor that only becomes
known at runtime doesn't get that optimization and pays the software
division cost.

## Usage examples (Embedded)

### Averaging accumulated ADC samples in place

```
let mut sum: u32 = adc_samples.iter().map(|&s| s as u32).sum();
sum /= adc_samples.len() as u32; // <- `/=` divides in place; a software routine unless the divisor is a compile-time power of two
```

### Choosing a power-of-two sample count so `/=` compiles to a shift

```
const SAMPLE_COUNT: u32 = 8; // power of two: `/=` by this constant compiles to a shift, not a divide

fn average_in_place(sum: &mut u32) {
    *sum /= SAMPLE_COUNT; // <- `/=` here is free: division by a power-of-two constant lowers to a shift
}
```
