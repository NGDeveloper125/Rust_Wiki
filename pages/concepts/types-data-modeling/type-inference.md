---
title: "Type inference"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Coming from Python / JavaScript"]
related_syntax: [let]
see_also: []
---

## Explanation

Rust infers a variable's type from how it's used, rather than requiring
an explicit annotation everywhere — `let x = 5;` doesn't need
`let x: i32 = 5;` unless the surrounding context is genuinely ambiguous.

This is a form of *local* type inference: every value still has exactly
one concrete, static type, decided entirely at compile time — Rust is not
dynamically typed, and there's no runtime type tag or type check the way
Python or JavaScript variables carry one. Inference just means you don't
always have to *write* the type for the compiler to know it; it works the
type out from the initializer expression, later usage, and function
signatures it flows into.

Because inference is purely a compile-time convenience and never changes
what's possible at runtime, it's fully compatible with Rust's zero-cost
philosophy — inferred code compiles to exactly the same thing as if every
type had been spelled out by hand. Function signatures are a deliberate
exception: parameter and return types must always be written explicitly,
which keeps a function's public contract readable and stable without
having to read its body to know what it accepts and returns.

## Basic usage example

```
let x = 5; // <- inferred as i32 from context (the default integer type)

let mut v = Vec::new();
v.push(3.14); // <- this later use tells the compiler v: Vec<f64>
```

**Restriction:** inference is local to a function body — parameter and
return types must always be written explicitly, so
`fn largest<T: PartialOrd>(items: &[T]) -> &T` can't have its signature
worked out just from how the function is used.

## Best practices & deeper information

### Scenario: Creating a new object

`collect()` is generic over what it builds and has no default target
type, so when nothing downstream pins that down, inference has nothing
to resolve against — either a turbofish or an annotated binding has to
supply it.

```
let readings = ["21.5", "22.0", "19.8"];

// AVOID: the compiler can't tell which collection collect() should build here
// let parsed = readings.iter().map(|r| r.parse::<f64>().unwrap()).collect(); // <- ambiguous target type

// PREFER: turbofish pins the target type at the call site
let parsed = readings
    .iter()
    .map(|r| r.parse::<f64>().unwrap())
    .collect::<Vec<f64>>(); // <- turbofish disambiguates what collect() should build

// PREFER (equivalent): or let the binding's own annotation do the same job
let parsed_alt: Vec<f64> = readings // <- annotation on the binding instead of the call
    .iter()
    .map(|r| r.parse::<f64>().unwrap())
    .collect();
```

**Why this way:** the
[standard library's `collect()` docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect)
recommend exactly this — turbofish or an annotated binding — as the fix
whenever inference can't work out the target collection on its own,
which is common enough with `collect()` specifically that it's the
canonical example of when an explicit type is needed.

## Embedded Rust Notes

**Full support.** A compile-time-only mechanism — no `std` dependency,
identical behavior in `#![no_std]`.
