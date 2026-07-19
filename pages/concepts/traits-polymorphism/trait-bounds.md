---
title: "Trait bounds"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Writing Generic & Reusable Code", "Decoupling", "Generic Programming"]
related_syntax: [":", "+", where]
see_also: ["Traits", "Generics", "Supertraits"]
---

## Explanation

A trait bound constrains a generic type parameter to only the types that
implement a given trait, giving generic code exactly the guarantees it
needs to actually do something useful with a value of an otherwise
unknown type:

```
fn largest<T: PartialOrd>(items: &[T]) -> &T { ... }
```

Without the bound, the compiler would have no basis for allowing `>` to
be used on values of type `T` inside the function body — `T` could be
anything. The bound `T: PartialOrd` says "whatever `T` ends up being, it
must support ordering comparisons," which the compiler then checks holds
at every call site.

This is the mechanism that lets Rust decouple code from concrete types
in a fully type-checked way: a function can depend on "any type that can
be compared" or "any type that can be displayed," rather than a specific
concrete type, and the compiler verifies both that the generic code only
uses operations the bound actually grants, and that every real type
passed in at a call site genuinely satisfies the bound. Multiple bounds
combine with `+` (`T: Clone + Debug`), and a `where` clause is available
for bounds that get too long to read comfortably inline.

## Basic usage example

```
fn largest<T: PartialOrd>(items: &[T]) -> &T { // <- bound: only types supporting `>` allowed
    let mut m = &items[0];
    for item in items {
        if item > m { m = item; }
    }
    m
}

largest(&[3, 7, 2]);
```

## Best practices & deeper information

### Scenario: Writing generic code

Once a generic function needs more than one bound, or the bound list gets
long, moving it to a `where` clause keeps the signature itself scannable
without changing what's required.

```
fn unique_sorted<T>(readings: &[T]) -> Vec<T>
where
    T: Ord + Clone, // <- bounds relocated to `where`, still constraining T the same way
{
    let mut sorted: Vec<T> = readings.to_vec(); // needs Clone
    sorted.sort();                              // needs Ord
    sorted.dedup();
    sorted
}

unique_sorted(&[3, 1, 3, 2, 1]);
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch10-02-traits.html)
recommends `where` once a function has multiple generic parameters each
carrying their own bounds — it separates "what the function does" (the
signature) from "what its inputs must support" (the bounds).

### Scenario: Designing a public API

A generic function should only require what its own body actually uses —
bounding it against a broader trait than necessary forces every caller's
type to satisfy requirements the function never touches.

```
// AVOID: over-constrained — Clone is required but never used
fn describe_avoid<T: Clone + std::fmt::Debug>(item: &T) -> String {
    format!("{item:?}")
}

// PREFER: bound only what the body needs
fn describe<T: std::fmt::Debug>(item: &T) -> String { // <- Debug is the only bound this fn needs
    format!("{item:?}")
}
```

**Why this way:** minimal bounds keep the function usable by the widest
range of types and make the signature an honest description of its
requirements — the
[API Guidelines' C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html)
names this directly: functions should minimize assumptions about their
parameters.

## Embedded Rust Notes

**Full support.** A purely compile-time mechanism — no `std`/allocator
dependency, and central to how `embedded-hal`-based drivers stay generic
over any concrete peripheral implementation.
