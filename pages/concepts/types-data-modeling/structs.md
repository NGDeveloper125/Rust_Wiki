---
title: "Structs"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Object-Oriented-ish Patterns", "Designing Robust Data Models", "Composition"]
related_syntax: [struct]
see_also: ["Tuple structs", "Unit structs", "Enums (algebraic data types)"]
---

## Explanation

A struct groups related values together under named fields into a single
type — the basic building block for modeling data with meaning attached
to each piece, rather than passing several loose, unrelated parameters
around.

```
struct Point {
    x: f64,
    y: f64,
}
```

Structs are Rust's primary way to give a related bundle of data its own
identity and its own methods (via `impl` blocks), filling a role similar
to a class in an object-oriented language but without inheritance —
behavior is attached through trait implementations and composition (a
struct containing other structs) rather than an inheritance hierarchy.
This composition-first approach is a deliberate design choice: nesting
structs inside each other, and implementing shared traits across
otherwise-unrelated types, covers most of what inheritance is used for in
other languages, without the fragility multi-level inheritance
hierarchies tend to accumulate over time.

## Basic usage example

```
struct Point { // <- defines the struct: a new type with two named fields
    x: f64,
    y: f64,
}

let p = Point { x: 1.0, y: 2.0 }; // constructing a value of that type
println!("{}", p.x); // fields are accessed by name
```

## Embedded Rust Notes

**Full support.** Structs are core-language and allocator-free — the
primary way embedded HAL crates model peripherals, register blocks, and
driver state.
