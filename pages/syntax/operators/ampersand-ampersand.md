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

## Embedded Rust Notes

**Full support.** Built into the language, not a trait — no `std`
dependency.
