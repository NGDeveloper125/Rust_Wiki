---
title: "Type aliases"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling"]
related_syntax: [type]
see_also: ["The newtype pattern"]
---

## Explanation

A type alias gives an existing type a new name, purely for readability —
it does not create a new, distinct type the way a
[newtype](the-newtype-pattern.md) does:

```
type Kilometers = f64;
let distance: Kilometers = 5.0;
let x: f64 = distance; // fine — Kilometers and f64 are the same type
```

Aliases are most valuable for shortening long, repeated type signatures
(especially generic ones, like `type Result<T> = std::result::Result<T, MyError>;`,
a very common pattern for a crate's own error type) and for giving
context to an otherwise-anonymous type in a signature. Because an alias
is fully interchangeable with what it aliases, it provides zero type
safety benefit on its own — if the goal is to prevent two `f64` values
that mean different things from being mixed up, a
[newtype](the-newtype-pattern.md) (an actual distinct type) is the tool
for that, not a type alias.

## Embedded Rust Notes

**Full support.** Purely a compile-time naming convenience — no `std`
dependency.
