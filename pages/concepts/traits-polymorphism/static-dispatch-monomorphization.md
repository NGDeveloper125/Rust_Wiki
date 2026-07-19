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

## Embedded Rust Notes

**Full support.** No allocator dependency — and often the preferred
choice in embedded code specifically to avoid `dyn Trait`'s vtable
indirection, at the cost of larger compiled binary size (a real, often
tighter constraint on embedded flash than on a hosted target).
