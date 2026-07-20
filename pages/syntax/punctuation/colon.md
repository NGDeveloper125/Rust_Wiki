---
title: ":"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Traits, Generics]
related_syntax: [let, fn]
see_also: []
---

## Explanation

`:` introduces a type or constraint annotation. Its exact meaning depends
entirely on position:

- **Variable/parameter type:** `let x: i32 = 5;`, `fn f(x: i32)`
- **Struct field initializer:** `Point { x: 1, y: 2 }`
- **Trait bound:** `fn f<T: Clone>(x: T)` — "`T` must implement `Clone`"
- **Loop label:** `'outer: loop { ... }`
- **Match-arm-like key/value pairs** in some macros

`:` is not an operator and has no overloadable meaning — it's pure
grammar, marking "what follows describes/constrains what came before."
Compare with `::`, a completely different token (path separator) that
happens to look like two of these stacked, but is lexed as its own single
token, not as two colons.

## Basic usage example

```
let x: i32 = 5;
//   ^ `:` here separates the binding name from its type annotation
```

## Best practices & deeper information

### Scenario: Writing generic code

Stacking multiple bounds after `:` (`T: Clone + std::fmt::Debug`) reads
better than it sounds, but once a function needs several bounds on
several type parameters, moving them to a `where` clause keeps the
signature itself scannable.

```
fn summarize<T>(items: &[T]) -> String
where
    T: std::fmt::Debug + Clone, // <- `:` here still constrains T, just relocated
{
    format!("{items:?}")
}
```

**Why this way:** the
[Book's chapter on trait bounds](https://doc.rust-lang.org/book/ch10-02-traits.html#clearer-trait-bounds-with-where-clauses)
recommends `where` once a bound list grows past one or two simple traits
— it keeps the parameter list itself readable and puts every constraint
in one predictable place.

### Scenario: Creating a new object

In a struct literal, `field: value` pairs use the same `:` token as a
type annotation but mean something different — "this field gets this
value," not "this name has this type."

```
struct Point { x: f64, y: f64 } // <- `:` here is a type annotation

let origin = Point { x: 0.0, y: 0.0 }; // <- `:` here is a field initializer
let shifted = Point { x: 1.0, ..origin }; // shorthand: y taken from origin
```

**Why this way:** when a local variable already shares a field's name
(`let x = 0.0; Point { x, y: 0.0 }`), the field-init shorthand drops the
`: value` entirely — `rustfmt`/clippy's
[`redundant_field_names`](https://rust-lang.github.io/rust-clippy/master/#redundant_field_names)
lint flags the redundant `field: field` form.

## Embedded Rust Notes

**Full support.** Pure grammar — no `std` dependency.
