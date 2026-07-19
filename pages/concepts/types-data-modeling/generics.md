---
title: "Generics"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Writing Generic & Reusable Code", "Polymorphism", "Generic Programming"]
related_syntax: ["<...>"]
see_also: ["Trait bounds", "Static dispatch & monomorphization", "Const generics"]
---

## Explanation

Generics let a type or function be written once and used with many
different concrete types, without duplicating the code for each one:

```
fn largest<T: PartialOrd>(items: &[T]) -> &T { ... }
```

Here `T` stands for "some type, to be determined at each call site,"
constrained by a [trait bound](../traits-polymorphism/trait-bounds.md)
(`PartialOrd`, in this example) so the function body can rely on the
operations it actually needs. Generics are resolved entirely at compile
time — see
[static dispatch & monomorphization](../traits-polymorphism/static-dispatch-monomorphization.md)
— which means generic code has no inherent runtime overhead compared to
writing the same function by hand for each concrete type; the compiler
does that duplication for you, automatically, and specializes each copy
for its specific type.

This is the main way Rust achieves reusable, type-safe abstractions
without needing to fall back to dynamic typing or runtime type checks —
the compiler verifies, once, at the definition site plus every call site,
that every operation the generic code performs is valid for whatever
concrete type ends up substituted in.

## Embedded Rust Notes

**Full support.** Generics and monomorphization are purely a compile-time
mechanism — no `std` or allocator dependency, and no extra binary size
beyond what monomorphization already costs on any target.
