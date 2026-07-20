---
title: ">="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["<", "<=", ">"]
see_also: [">"]
---

## Explanation

`>=` is the greater-than-or-equal comparison, part of the same
`std::cmp::PartialOrd` trait as `<`, `<=`, and `>`.

## Basic usage example

```
let a = 5;
let b = 3;
let ok = a >= b; // <- true if `a` is greater than or equal to `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a >= b >= c` doesn't compile; write `a >= b && b >= c` instead.

## Best practices & deeper information

### Scenario: Validating input

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

**Why this way:** naming `MIN_CREDIT_SCORE` gives the boundary one place
to update and makes the `>=` comparison read as "at least this much"
rather than a bare number whose meaning has to be inferred — see
[`<`](less-than.md) for the fuller treatment of ordering comparisons in
general.

## Embedded Rust Notes

**Full support.** `PartialOrd` lives in `core::cmp` — no `std` dependency.
