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
[newtype](the-newtype-pattern.md) does. `type Kilometers = f64;` is a
typical example: a value declared as `Kilometers` is directly
interchangeable with one declared as plain `f64`.

Aliases are most valuable for shortening long, repeated type signatures
(especially generic ones, like `type Result<T> = std::result::Result<T, MyError>;`,
a very common pattern for a crate's own error type) and for giving
context to an otherwise-anonymous type in a signature. Because an alias
is fully interchangeable with what it aliases, it provides zero type
safety benefit on its own — if the goal is to prevent two `f64` values
that mean different things from being mixed up, a
[newtype](the-newtype-pattern.md) (an actual distinct type) is the tool
for that, not a type alias.

## Basic usage example

```
type Kilometers = f64; // <- just another name for f64, not a distinct type

let distance: Kilometers = 5.0;
let x: f64 = distance; // <- fine: Kilometers and f64 are the same type
```

**Restriction:** an alias provides no type safety — it's fully
interchangeable with what it aliases, so the compiler won't catch two
aliases of the same underlying type being mixed up (unlike a
[newtype](the-newtype-pattern.md)).

## Best practices & deeper information

### Scenario: Designing a public API

An alias earns its place when a long, repeated generic type would
otherwise clutter every signature it appears in — but it's worth being
honest that it buys readability only, nothing more.

```
use std::collections::HashMap;

type SensorIndex = HashMap<String, Vec<u32>>; // <- pure readability: SensorIndex IS a HashMap, nothing new

fn build_index(entries: &[(String, u32)]) -> SensorIndex { // <- reads far better than the full HashMap<...> type
    let mut index: SensorIndex = HashMap::new();
    for (name, reading) in entries {
        index.entry(name.clone()).or_default().push(*reading);
    }
    index
}

// but the alias gives zero type safety: this still compiles, alias or not --
let raw: HashMap<String, Vec<u32>> = build_index(&[]); // <- SensorIndex and this HashMap are the same type
```

**Why this way:** an alias is fully interchangeable with what it
aliases, so it's the right tool purely for shortening a long,
repeated signature — the moment the goal is preventing two values that
happen to share an underlying type from being mixed up, that's a job for
[the newtype pattern](the-newtype-pattern.md) instead, which actually
creates a new, distinct type.

## Embedded Rust Notes

**Full support.** Purely a compile-time naming convenience — no `std`
dependency.
