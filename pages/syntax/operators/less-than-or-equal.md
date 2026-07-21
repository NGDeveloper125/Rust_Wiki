---
title: "<="
kind: operator
embedded_support: full
groups: [Comparison, Basics]
related_concepts: [Operator overloading]
related_syntax: ["<", ">", ">="]
see_also: ["<"]
---

## Explanation

`<=` is the less-than-or-equal comparison, provided by `std::cmp::PartialOrd`
alongside `<`, `>`, and `>=` — implementing `PartialOrd` (usually via
`#[derive(PartialOrd)]`, which requires `PartialEq` as well) gives you all
four ordering operators together, not just one.

## Basic usage example

```
let a = 3;
let b = 5;
let ok = a <= b; // <- true if `a` is less than or equal to `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a <= b <= c` doesn't compile; write `a <= b && b <= c` instead.

## Best practices & deeper information

### Scenario: Validating input

Writing an inclusive range check as `MIN <= x && x <= MAX` mirrors how
the range reads on paper, keeping both endpoints explicitly included.

```
struct Reading {
    celsius: f64,
}

const MIN_SAFE: f64 = -20.0;
const MAX_SAFE: f64 = 60.0;

fn in_safe_range(reading: &Reading) -> bool {
    MIN_SAFE <= reading.celsius && reading.celsius <= MAX_SAFE // <- `<=` twice expresses an inclusive range
}
```

**Why this way:** `MIN <= x && x <= MAX` keeps both bounds inclusive
explicitly, rather than reaching for a `Range` (`MIN..MAX`, which is
half-open) where an inclusive upper bound is actually what's intended.
The idiomatic form is `(MIN..=MAX).contains(&x)` — clippy's
[`manual_range_contains`](https://rust-lang.github.io/rust-clippy/master/index.html#manual_range_contains)
lint suggests exactly that rewrite — but the explicit `&&` chain remains
fine when you want the bounds visible inline; see [`<`](less-than.md)
for the fuller ordering-operator treatment.

## Embedded Rust Notes

**Full support.** `PartialOrd` lives in `core::cmp` — no `std` dependency.
