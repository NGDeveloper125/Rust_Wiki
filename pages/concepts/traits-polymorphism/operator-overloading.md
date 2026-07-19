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

## Best practices & deeper information

### Scenario: Creating a new object

Combining two `Money` values into a new one reads naturally as `a + b`
once `Add` is implemented — the operator becomes the constructor for the
summed value.

```
use std::ops::Add;

#[derive(Clone, Copy)]
struct Money { cents: i64 }

impl Add for Money { // <- gives `+` its meaning: combining two Money values into a new one
    type Output = Money;
    fn add(self, other: Money) -> Money {
        Money { cents: self.cents + other.cents }
    }
}

let total = Money { cents: 500 } + Money { cents: 250 }; // <- builds a new Money via `+`
```

**Why this way:**
[`std::ops::Add`](https://doc.rust-lang.org/std/ops/trait.Add.html) exists
precisely so a value can be combined with `+` instead of a
differently-named method like `.plus()` — once implemented, `Money`
composes with any code already written in terms of `+`.

### Scenario: Designing a public API

Operator overloading is idiomatic only when the operator's usual meaning
genuinely applies — implementing `Add` for something that isn't really
"addable" surprises every caller who reads `a + b` and expects
arithmetic-like behavior.

```
struct Config { retries: u32 }

// AVOID: overloads `+` but the meaning is surprising (not addition)
impl std::ops::Add for Config {
    type Output = Config;
    fn add(self, other: Config) -> Config {
        Config { retries: self.retries.max(other.retries) }
    }
}

// PREFER: a named method when the operation isn't actually arithmetic
impl Config {
    fn merged_with(self, other: Config) -> Config { // <- no operator trait involved
        Config { retries: self.retries.max(other.retries) }
    }
}
```

**Why this way:** the
[API Guidelines' C-OVERLOAD](https://rust-lang.github.io/api-guidelines/predictability.html)
is explicit that operators come with strong expectations — implement
`Add` "only for an operation that bears some resemblance to addition,"
and reach for a named method otherwise.

## Embedded Rust Notes

**Full support.** All `std::ops` traits live in `core::ops` — no `std`
dependency.
