---
title: "Static dispatch & monomorphization"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Polymorphism"]
related_syntax: []
see_also: ["Trait objects & dynamic dispatch (dyn Trait)", "Generics", "Zero-cost abstractions"]
---

## Explanation

When generic code is compiled, the compiler generates a separate,
specialized copy of it for every concrete type it's actually called with
— this is monomorphization ("making one shape from many"). Calling
`largest::<i32>(...)` and `largest::<String>(...)` in the same program
produces two distinct compiled functions, each specialized to its type,
with the choice of which one to call baked in and resolved entirely at
compile time — no runtime lookup, no indirection.

This is "static" dispatch: the exact function being called is known
statically, at compile time, as opposed to
[dynamic dispatch](trait-objects-dynamic-dispatch.md) (`dyn Trait`),
where the specific implementation is chosen at runtime via a vtable
lookup. The tradeoff is binary size versus runtime cost: monomorphization
can produce larger compiled binaries (one copy per concrete type used),
but each copy runs exactly as fast as if you'd hand-written it for that
specific type — this is precisely what "zero-cost abstraction" means in
Rust's design: the abstraction (writing generic code once) costs nothing
at runtime compared to the non-abstracted equivalent (writing each
specialized version by hand yourself).

## Basic usage example

```
fn largest<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

largest(1, 2);         // <- compiler generates a largest::<i32> copy
largest("a", "b");     // <- ...and a separate largest::<&str> copy, chosen at compile time
```

## Best practices & deeper information

### Scenario: Writing generic code

A generic function called with a small, known set of concrete types is a
good candidate for static dispatch — each call site gets its own
specialized, inlinable copy with no vtable indirection.

```
fn largest<T: PartialOrd>(a: T, b: T) -> T { // <- generic: monomorphized per concrete type used
    if a > b { a } else { b }
}

largest(3, 7);       // compiler emits a largest::<i32> copy
largest(3.5, 2.1);   // ...and a separate largest::<f64> copy, both resolved at compile time
```

**Why this way:** static dispatch trades binary size for speed — no
runtime lookup, and each copy can be inlined and optimized as if
hand-written — which
[Effective Rust's item on generics vs. trait objects](https://effective-rust.com/generics.html)
recommends as the default choice; reach for
[trait objects & dynamic dispatch](trait-objects-dynamic-dispatch.md)
instead once the set of concrete types is only known at runtime or code
size becomes the binding constraint.

## Embedded Rust Notes

**Full support.** No allocator dependency — and often the preferred
choice in embedded code specifically to avoid `dyn Trait`'s vtable
indirection, at the cost of larger compiled binary size (a real, often
tighter constraint on embedded flash than on a hosted target).
