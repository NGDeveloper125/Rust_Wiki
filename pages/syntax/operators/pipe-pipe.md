---
title: "||"
kind: operator
embedded_support: full
groups: [Basics, "Functions & Closures"]
related_concepts: []
related_syntax: ["&&", "|"]
see_also: ["&&"]
---

## Explanation

`||` is short-circuiting logical OR between two `bool` values:

```
if a < 0 || b < 0 { ... }
```

The right operand only evaluates if the left is `false`. Like `&&`, `||`
is not overloadable — always `bool`, always short-circuiting.

`||` also opens and closes a **zero-argument closure**'s parameter list:

```
let f = || println!("called");
```

This is the same `|...|` closure syntax as `|x, y| x + y`, just with an
empty parameter list — the parser distinguishes the two uses by context,
since a bare `||` in expression position where a value (not a boolean
condition) is expected can only be a closure.

## Basic usage example

```
let out_of_range = a < 0 || a > 100; // <- `||` short-circuiting logical OR
```

## Embedded Rust Notes

**Full support** for both meanings — logical OR and zero-argument
closures both work identically in `#![no_std]` (closures don't require
heap allocation unless they're boxed as `dyn Fn`).
