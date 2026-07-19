---
title: "!="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["=="]
see_also: ["=="]
---

## Explanation

`!=` tests inequality — the negation of `==`. It's provided automatically
by `std::ops::PartialEq` (a single trait supplies both `eq` and, by
default, `ne` as `!self.eq(other)`); a type essentially never needs to
implement `!=` separately from `==`.

```
if a != b { ... }
```

## Basic usage example

```
let ready = state != 0; // <- `!=` tests for inequality
```

## Embedded Rust Notes

**Full support.** Same trait as [`==`](equal-equal.md), no `std`
dependency.
