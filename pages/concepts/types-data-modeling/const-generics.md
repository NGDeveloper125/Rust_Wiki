---
title: "Const generics"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Declarative / Metaprogramming", "Writing Generic & Reusable Code", "Unique to Rust", "Generic Programming", "Coming from C / C++"]
related_syntax: [const]
see_also: ["Generics", "Associated types"]
---

## Explanation

Const generics parameterize a type by a *value*, not just another type —
most commonly an array length:

```
struct Buffer<const N: usize> {
    data: [u8; N],
}
let a: Buffer<64> = Buffer { data: [0; 64] };
```

Before const generics existed, array length wasn't something generic code
could abstract over at all — `[T; N]` for different `N` were unrelated
types with no shared generic interface, forcing either code duplication
per size or falling back to a heap-allocated `Vec` even when a fixed size
was known and stack allocation would have been possible and faster. Const
generics close that gap: `N` is checked and resolved entirely at compile
time, the same way a type parameter is, so `Buffer<64>` and `Buffer<128>`
are monomorphized into separate, specialized code paths with no runtime
cost for the abstraction.

This maps closely to what C++ templates have long allowed with
non-type template parameters, but with Rust's stricter compile-time
checking of what operations on `N` are actually valid.

## Basic usage example

```
fn sum<const N: usize>(arr: [i32; N]) -> i32 { // <- N is a value, known at compile time
    arr.iter().sum()
}

sum([1, 2, 3]);      // N = 3, inferred from the array literal
sum([1, 2, 3, 4]);    // N = 4, a distinct monomorphized instantiation
```

**Restriction:** `N` must be resolvable at compile time — it can be a
literal, a `const`, or inferred from context, but never a value computed
at runtime (like a `Vec`'s length).

## Embedded Rust Notes

**Full support.** No allocator dependency — const generics are
especially valuable in embedded/`no_std` code, since they let
fixed-capacity, stack-allocated buffer types (like `heapless::Vec<T, N>`)
express their capacity in the type system instead of needing heap
allocation at all.
