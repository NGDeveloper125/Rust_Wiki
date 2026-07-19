---
title: "in"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: ["The Iterator trait"]
related_syntax: [for]
see_also: [for]
---

## Explanation

`in` binds the loop variable to the iterator source in a `for` loop:

```
for x in 0..10 { ... }
```

Outside of `for ... in ...`, `in` has no independent meaning as an
operator or standalone keyword — it exists solely as part of the `for`
loop's fixed grammar (`for PATTERN in EXPR { BLOCK }`), not as a general
membership-test operator the way `in` works in Python. Testing membership
in a Rust collection is a method call instead
(`collection.contains(&value)`).

## Basic usage example

```
for x in 0..10 { // <- `in` binds `x` to each value produced by `0..10`
    println!("{x}");
}
```

**Restriction:** `in` only exists as part of the fixed
`for PATTERN in EXPR { ... }` grammar — it is not a standalone
membership-test operator the way `in` works in Python.

## Embedded Rust Notes

**Full support.** Pure grammar, part of the `for` loop form — no `std`
dependency.
