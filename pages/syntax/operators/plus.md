---
title: "+"
kind: operator
embedded_support: full
groups: [Basics, "Traits & Polymorphism"]
related_concepts: [Operator overloading]
related_syntax: ["+="]
see_also: ["+="]
---

## Explanation

`+` is arithmetic addition between two values of the same numeric type:

```
let sum = 1 + 2;
```

It's overloadable via `std::ops::Add` — any type can define what `+`
means for it, which is how `String + &str` concatenation works (`Add` is
implemented for `String`, consuming the left operand by value).

`+` also has a completely unrelated meaning in **trait-bound position**,
where it combines multiple bounds/lifetimes rather than performing
arithmetic:

```
fn f<T: Clone + Debug>(x: T) { ... }
fn g(x: &(dyn Trait + Send)) { ... }
```

Here `+` reads as "and" — `T` must implement both `Clone` and `Debug`;
the trait object must implement `Trait` and be `Send`. This is pure
compile-time grammar with no `Add`-trait involvement at all; don't
confuse the two uses.

## Basic usage example

```
let sum = 1 + 2; // <- `+` adds two values
```

## Embedded Rust Notes

**Full support.** `Add` lives in `core::ops` (re-exported as `std::ops`),
so both the arithmetic and trait-bound-combinator meanings work
identically in `#![no_std]`.
