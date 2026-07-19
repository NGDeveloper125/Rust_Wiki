---
title: "Arrays vs Vec"
area: "Types & Data Modeling"
embedded_support: partial
groups: ["Types & Data Modeling", "Working with Collections", "Collections"]
related_syntax: ["[ ]", ";"]
see_also: ["Slices"]
---

## Explanation

An array, `[T; N]`, has a fixed length baked into its type — `[i32; 3]`
and `[i32; 5]` are different types entirely — and is allocated inline
(on the stack, if the array itself is a local variable) with no
heap allocation or indirection involved. `Vec<T>` is a growable,
heap-allocated sequence whose length is tracked at runtime and can change
over the value's lifetime (`.push()`, `.pop()`, `.remove()`, …).

The choice between them is really a choice about whether the length is
known and fixed at compile time. If it is (a 3D vector's `[f64; 3]`
components, a fixed-size lookup table), an array avoids heap allocation
entirely and is typically faster and more cache-friendly. If the length
varies at runtime (user input, parsed data, anything built up
incrementally), `Vec` is the only one of the two that can represent that
at all.

Both deref to a [slice](slices.md) (`&[T]`) for reading, which is why
functions that just need to *read* a sequence, regardless of its backing
storage, conventionally take `&[T]` rather than committing to either
`&Vec<T>` or `&[T; N]` specifically.

## Embedded Rust Notes

**Partial support.** Fixed-size arrays (`[T; N]`) are full support — pure
`core`, stack-allocated, no allocator needed, and the default choice in
embedded code for exactly that reason. `Vec<T>` itself lives in `alloc`
and needs a configured `#[global_allocator]`; where growable
storage is needed without a heap, `heapless::Vec<T, N>` provides a
fixed-*capacity* (but runtime-variable-*length*) alternative with no
allocator dependency at all.
