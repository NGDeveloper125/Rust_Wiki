---
title: "=="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading, "Derivable traits (Debug, Clone, PartialEq, …)"]
related_syntax: ["!="]
see_also: ["!="]
---

## Explanation

`==` tests equality, overloadable via `std::ops::PartialEq` (usually
obtained via `#[derive(PartialEq)]` rather than hand-written):

```
if a == b { ... }
```

`PartialEq` is "partial" because equality need not be total — floating
point `NaN != NaN`, which is why `f32`/`f64` implement `PartialEq` but not
the stricter `Eq` (which additionally requires reflexivity: `x == x`
always). Comparing two values whose type doesn't implement `PartialEq` is
a compile error, not a runtime failure — there's no default "compare by
reference identity" fallback the way some languages have.

## Basic usage example

```
let a = 5;
let b = 5;
let same = a == b; // <- `==` compares `a` and `b` for equality
```

**Restriction:** `==` can't be chained — `a == b == c` doesn't compile,
since the `bool` result of `a == b` doesn't implement `PartialEq<T>` for
an arbitrary `T`.

## Embedded Rust Notes

**Full support.** `PartialEq` lives in `core::cmp` — no `std` dependency.
