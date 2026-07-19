---
title: "Operator overloading (std::ops traits)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Polymorphism"]
related_syntax: ["+", "*", "==", "[ ]"]
see_also: ["Traits"]
---

## Explanation

Operators like `+`, `==`, `[]`, and `*` aren't special-cased into the
language for user-defined types — they're ordinary trait methods,
defined in `std::ops` (`Add`, `PartialEq`, `Index`, `Mul`, and so on),
which any type can implement to give meaning to that operator for itself:

```
impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}
```

Once implemented, `p1 + p2` calls this `add` method directly — the
operator syntax is just sugar over the trait method call, resolved at
compile time. Because it's an ordinary trait, the same rules apply as to
any other trait: you can only implement it for types you own (or foreign
traits on foreign types via [the newtype pattern](../types-data-modeling/the-newtype-pattern.md)),
and the compiler enforces that the types involved actually make sense
together (e.g. `Add`'s associated `Output` type must be specified
explicitly, so `Point + Point` returning something other than `Point` is
possible but has to be deliberate).

## Basic usage example

```
use std::ops::Add;

struct Point { x: i32, y: i32 }

impl Add for Point { // <- gives `+` its meaning for Point
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

let p = Point { x: 1, y: 2 } + Point { x: 3, y: 4 }; // <- calls Point::add
```

## Embedded Rust Notes

**Full support.** All `std::ops` traits live in `core::ops` — no `std`
dependency.
