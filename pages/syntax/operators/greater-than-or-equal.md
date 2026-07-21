---
title: ">="
kind: operator
embedded_support: full
groups: [Comparison, Basics]
related_concepts: [Operator overloading]
related_syntax: ["<", "<=", ">"]
see_also: [">"]
---

## Explanation

`>=` is the greater-than-or-equal comparison, part of the same
`std::cmp::PartialOrd` trait as `<`, `<=`, and `>`.

## Usage examples

### Checking a greater-than-or-equal comparison

```
let a = 5;
let b = 3;
let ok = a >= b; // <- true if `a` is greater than or equal to `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a >= b >= c` doesn't compile; write `a >= b && b >= c` instead.

### Validating input

`>=` enforces an inclusive minimum, and naming the threshold as a
constant rather than inlining a magic number keeps the check
self-documenting.

```
struct Applicant {
    age: u32,
    credit_score: u32,
}

const MIN_CREDIT_SCORE: u32 = 650;

fn is_eligible(applicant: &Applicant) -> bool {
    applicant.age >= 18 && applicant.credit_score >= MIN_CREDIT_SCORE // <- `>=` enforces each inclusive minimum
}
```

Naming `MIN_CREDIT_SCORE` gives the boundary one place
to update and makes the `>=` comparison read as "at least this much"
rather than a bare number whose meaning has to be inferred — see
[`<`](less-than.md) for the fuller treatment of ordering comparisons in
general.

## Explanation (Embedded)

`>=` means the same thing under `#![no_std]` — same `core::cmp`
`PartialOrd` as `>`, `<`, and `<=`. The inclusive-lower-bound reading is
genuinely common in firmware: a battery voltage that has reached (not
just exceeded) a "sufficiently charged" threshold, or a fixed-size
buffer's write index that has reached exactly its capacity — in both
cases the boundary value itself is meant to count, which is what makes
`>=` the right comparison instead of `>`.

## Usage examples (Embedded)

### Checking a battery voltage has reached the operating threshold

```
const MIN_OPERATING_MILLIVOLTS: u16 = 3300;

fn battery_ok(voltage_millivolts: u16) -> bool {
    voltage_millivolts >= MIN_OPERATING_MILLIVOLTS // <- `>=` treats the threshold itself as still acceptable
}
```

### Reporting a fixed-size buffer as full

```
const BUFFER_CAPACITY: usize = 64;

fn buffer_is_full(write_index: usize) -> bool {
    write_index >= BUFFER_CAPACITY // <- `>=` catches "exactly full" as well as "somehow past capacity"
}
```
