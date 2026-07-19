---
title: "&&"
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: ["||", "!"]
see_also: ["||"]
---

## Explanation

`&&` is short-circuiting logical AND between two `bool` values:

```
if a > 0 && b > 0 { ... }
```

"Short-circuiting" means the right operand is only evaluated if the left
is `true` — important when the right side has side effects or could
panic (`x.is_some() && x.unwrap() > 0`). Unlike `&`, `&&` is **not**
overloadable — it only ever works on `bool` and always short-circuits;
there's no trait to implement to change its behavior for a custom type.

## Basic usage example

```
let a = 3;
let b = 5;
let both_positive = a > 0 && b > 0; // <- `&&` short-circuits: `b > 0` only runs if `a > 0` is true
```

**Restriction:** `&&` only works on `bool` operands and can't be
overloaded for custom types — unlike `&`, there is no trait to implement
to change its behavior.

## Embedded Rust Notes

**Full support.** Built into the language, not a trait — no `std`
dependency.
