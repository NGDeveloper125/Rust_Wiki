---
title: "*="
kind: operator
embedded_support: full
groups: [Arithmetic, Basics]
related_concepts: [Operator overloading]
related_syntax: ["*"]
see_also: ["*"]
---

## Explanation

`*=` multiplies the left operand by the right in place, as in `x *= 2`,
and is overloadable via `std::ops::MulAssign`.

Unrelated to the dereference sense of `*` — `*=` is purely the compound
arithmetic-assignment operator; there is no "deref-assign" reading of
this token.

## Usage examples

### Multiplying a value in place

```
let mut x = 5;
x *= 3; // <- multiplies `x` by 3 in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `*=` assigns in place.

### Numeric computation

Scaling a running value repeatedly — compounding growth over several
steps — is a natural fit for `*=`, updating the value in place each
iteration instead of rebinding it.

```
let mut balance = 1000.0_f64;
let growth_rate = 1.05;

for _year in 0..5 {
    balance *= growth_rate; // <- multiplies `balance` by the rate in place, every iteration
}

println!("after 5 years: {balance:.2}");
```

`*=` makes the "scale this value repeatedly" intent
explicit at the call site instead of writing `balance = balance *
growth_rate` each time — see [`+=`](plus-equals.md) for the general notes
shared by every compound-assignment operator (mutable place required, its
own trait impl distinct from the plain operator).

## Explanation (Embedded)

`*=` means exactly the same thing under `#![no_std]` — `MulAssign` lives
in `core::ops`, with no `std` dependency to swap out. There's no
dereference angle to reconsider here the way there is for plain
[`*`](asterisk.md): this token is purely the compound multiplication
assignment. The one embedded-relevant nuance it shares with the rest of
the arithmetic family ([`+=`](plus-equals.md)/[`-=`](minus-equals.md)) is
overflow behavior: a release build — the profile that actually ships to a
device — has overflow checks off by default, so a `*=` that overflows
wraps silently instead of panicking, and a deployed board can't "just
recompile in debug" to catch it after the fact. Scaling a small,
bounded value in place is fine with bare `*=`; scaling something that
could plausibly grow past its type's range calls for `checked_mul`/
`wrapping_mul` assigned back explicitly.

## Usage examples (Embedded)

### Scaling a duty-cycle value in place

```
let mut duty: u16 = 200;
duty *= 2; // <- `*=` scales `duty` in place; silently wraps in release if this ever overflows u16
```

### Guarding a gain multiplier against release-mode wraparound

```
fn apply_gain(sample: u16, gain: u16) -> Option<u16> {
    let mut result = sample;
    result = result.checked_mul(gain)?; // stands in for a bare `*=` that could wrap unnoticed on a shipped device
    Some(result)
}
```
