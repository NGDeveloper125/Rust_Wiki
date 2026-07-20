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
by each specific implementation rather than chosen by the caller — for
instance, the standard `Iterator` trait declares `type Item;` and returns
`Option<Self::Item>` from `next`, without fixing what `Item` actually is
until a concrete type implements it.

Every type implementing `Iterator` picks exactly one concrete `Item` type
— the by-value iterator from `Vec<i32>` has `Item = i32`, and
`HashMap<K, V>`'s has `Item = (K, V)` (the borrowing `.iter()` yields
`&i32` and `(&K, &V)` respectively, each its own iterator type) — and that
choice is fixed for that implementation, unlike a generic type parameter,
which a caller could instantiate differently at each use site.

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

## Best practices & deeper information

### Scenario: Implementing traits

When a trait method's result type has exactly one right answer per
implementer, fixing it as an associated type keeps the trait's signature
simple and keeps callers from having to specify a type parameter that was
never theirs to choose.

```
trait Parser {
    type Output;                     // <- fixed per implementer, not chosen by the caller
    fn parse(&self, input: &str) -> Self::Output;
}

struct IntParser;
impl Parser for IntParser {
    type Output = i32;               // <- IntParser always produces i32, never anything else
    fn parse(&self, input: &str) -> i32 {
        input.parse().unwrap_or(0)
    }
}

// A generic `trait Parser<Output> { ... }` would instead let one type implement
// Parser<i32> AND Parser<String> at once -- rarely the right shape for
// "the type this parser produces," which should have exactly one answer.
```

**Why this way:** use an associated type when there's exactly one correct
answer per implementer (an iterator's `Item`, a parser's `Output`);
reach for a generic parameter instead when a caller legitimately needs to
choose it at the call site, the way a
[trait bound](../traits-polymorphism/trait-bounds.md) like `From<T>` lets
one type convert from many different `T`s.

## Embedded Rust Notes

**Full support.** A compile-time mechanism — no `std`/allocator
dependency. The `embedded-hal` ecosystem's traits make heavy use of
associated types (e.g. an error type associated with a given peripheral
trait) to stay generic across many vendors' hardware.
