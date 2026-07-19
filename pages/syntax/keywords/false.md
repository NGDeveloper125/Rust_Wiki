---
title: "false"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [true]
see_also: [true]
---

## Explanation

`false` is the boolean literal for a false value, of type `bool`. Like
`true`, it is both a reserved keyword and a complete literal expression.

```
let done: bool = false;
```

See [`true`](true.md) for the surrounding notes on `bool` as a distinct,
non-numeric type with no implicit conversions.

## Basic usage example

```
let done: bool = false; // <- `false` is the boolean literal for a false value
```

## Embedded Rust Notes

**Full support.** Same as [`true`](true.md) — a `core` primitive, no `std`
dependency.
