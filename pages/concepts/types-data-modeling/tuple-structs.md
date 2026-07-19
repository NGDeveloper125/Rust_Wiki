---
title: "Tuple structs"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling"]
related_syntax: [struct]
see_also: ["Structs", "The newtype pattern"]
---

## Explanation

A tuple struct is a struct whose fields are identified by position rather
than by name:

```
struct Point(f64, f64);
let p = Point(1.0, 2.0);
```

It's a middle ground between a plain tuple `(f64, f64)` (no type identity
of its own — any two `f64`s are interchangeable with it) and a full named
struct (every field has an explicit name). A tuple struct gives a set of
values a distinct type — `Point` is not the same type as `(f64, f64)` and
the two can't be used interchangeably — while keeping field access
positional (`p.0`, `p.1`) for cases where names would add little.

The single-field case, `struct Meters(f64);`, is common enough to have
its own name — see [the newtype pattern](the-newtype-pattern.md) — where
the point isn't brevity but using the type system to prevent mixing up
values that happen to share an underlying representation.

## Basic usage example

```
struct Point(f64, f64); // <- fields identified by position, not by name

let p = Point(1.0, 2.0);
println!("{}", p.0); // <- positional access: p.0, p.1
```

## Embedded Rust Notes

**Full support.** No allocator dependency — commonly used in embedded
HALs for lightweight typed wrappers (e.g. `struct Millivolts(u16);`).
