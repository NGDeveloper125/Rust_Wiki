---
title: "*="
kind: operator
embedded_support: full
groups: [Basics]
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

## Basic usage example

```
let mut x = 5;
x *= 3; // <- multiplies `x` by 3 in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `*=` assigns in place.

## Best practices & deeper information

### Scenario: Numeric computation

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

**Why this way:** `*=` makes the "scale this value repeatedly" intent
explicit at the call site instead of writing `balance = balance *
growth_rate` each time — see [`+=`](plus-equals.md) for the general notes
shared by every compound-assignment operator (mutable place required, its
own trait impl distinct from the plain operator).

## Embedded Rust Notes

**Full support.** `MulAssign` lives in `core::ops` — no `std` dependency.
