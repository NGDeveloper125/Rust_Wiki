---
title: "Derivable traits (Debug, Clone, PartialEq, …)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Serialization"]
related_syntax: [derive]
see_also: ["Traits", "Copy vs Clone"]
---

## Explanation

Many common traits have an obvious, mechanical implementation that can be
generated automatically from a type's structure — comparing a struct
field-by-field for `PartialEq`, printing every field for `Debug`, cloning
every field for `Clone`. `#[derive(...)]` generates exactly that
mechanical implementation, saving the boilerplate of writing it by hand:

```
#[derive(Debug, Clone, PartialEq)]
struct Point { x: f64, y: f64 }
```

A derive only works if every field's own type already implements the
trait being derived — deriving `Clone` for a struct containing a
non-`Clone` field is a compile error, since there'd be no way to generate
a working implementation. This is also the mechanism behind
`serde`'s `#[derive(Serialize, Deserialize)]` — the same
generate-from-structure idea, extended by a third-party procedural macro
rather than the small fixed set of derives built into the compiler
itself, which is why the [serialization](../../concepts/serialization.md)
ecosystem is able to plug into ordinary struct/enum definitions with a
single attribute rather than requiring hand-written conversion code.

## Basic usage example

```
#[derive(Debug, Clone, PartialEq)] // <- mechanically generates all three impls
struct Point { x: f64, y: f64 }

let a = Point { x: 1.0, y: 2.0 };
let b = a.clone();
println!("{:?} {}", b, a == b);
```

**Restriction:** a derive only works if every field's own type already
implements the trait being derived — a struct with a non-`Clone` field
cannot itself derive `Clone`.

## Embedded Rust Notes

**Full support.** The built-in derives (`Debug`, `Clone`, `PartialEq`,
etc.) all work in `#![no_std]`. Worth noting: `#[derive(Debug)]`'s
formatting still goes through `core::fmt`, which has no built-in way to
print anywhere on a bare-metal target — embedded code typically routes
`Debug`/`Display` output through a crate like `defmt` (a
`no_std`-oriented, wire-efficient logging framework) rather than
`println!`, which requires `std`.
