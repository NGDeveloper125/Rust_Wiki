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
most commonly an array length. For example, a
`Buffer<const N: usize> { data: [u8; N] }` struct can be instantiated as
`Buffer<64>`, with `N` becoming part of the type itself.

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

## Best practices & deeper information

### Scenario: Writing generic code

A fixed-size buffer type parameterized over its length lets the compiler
catch a capacity mismatch at compile time, and stack-allocate the backing
array, instead of needing a heap-allocated `Vec` just to carry a length
that's actually known up front.

```
struct RingBuffer<const N: usize> { // <- N is part of the type: RingBuffer<8> and RingBuffer<256> differ
    data: [u8; N],
    len: usize,
}

impl<const N: usize> RingBuffer<N> {
    fn new() -> Self {
        RingBuffer { data: [0; N], len: 0 }
    }

    fn capacity(&self) -> usize {
        N // <- N is an ordinary compile-time constant inside the impl, not a runtime field
    }
}

let small: RingBuffer<8> = RingBuffer::new();
let large: RingBuffer<256> = RingBuffer::new();
```

**Why this way:** encoding the capacity in the type itself means passing
a `RingBuffer<8>` where a `RingBuffer<256>` is expected is a compile
error rather than a bug discovered at runtime — the same compile-time
guarantee [generics](generics.md) give for types, extended to a value.

## Embedded Rust Notes

**Full support.** No allocator dependency — const generics are
especially valuable in embedded/`no_std` code, since they let
fixed-capacity, stack-allocated buffer types (like `heapless::Vec<T, N>`)
express their capacity in the type system instead of needing heap
allocation at all.
