---
title: "Associated types"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Writing Generic & Reusable Code", "Generic Programming"]
related_syntax: []
see_also: ["Generics", "Traits", "The Iterator trait"]
---

## Explanation

An associated type is a type placeholder attached to a trait, filled in
by each specific implementation rather than chosen by the caller:

```
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

Every type implementing `Iterator` picks exactly one concrete `Item` type
— `Vec<i32>`'s iterator has `Item = i32`, `HashMap<K, V>`'s has
`Item = (K, V)` — and that choice is fixed for that implementation,
unlike a generic type parameter, which a caller could instantiate
differently at each use site.

The distinction matters: if `Iterator` used a generic parameter instead
(`trait Iterator<Item> { ... }`), a single type could implement `Iterator<i32>`
*and* `Iterator<String>` simultaneously, which is rarely what you want for
something like "the type this iterator yields" — there's naturally
exactly one right answer per implementing type. Associated types express
that "exactly one, determined by the implementer" relationship directly,
while a generic parameter would leave it open to the caller in a way that
doesn't fit the intent.

## Basic usage example

```
trait Container {
    type Item;                 // <- associated type: each implementer fills this in
    fn get(&self, i: usize) -> Self::Item;
}

struct Numbers(Vec<i32>);

impl Container for Numbers {
    type Item = i32;           // <- this impl fixes Item to i32, exactly once
    fn get(&self, i: usize) -> i32 { self.0[i] }
}
```

## Embedded Rust Notes

**Full support.** A compile-time mechanism — no `std`/allocator
dependency. The `embedded-hal` ecosystem's traits make heavy use of
associated types (e.g. an error type associated with a given peripheral
trait) to stay generic across many vendors' hardware.
