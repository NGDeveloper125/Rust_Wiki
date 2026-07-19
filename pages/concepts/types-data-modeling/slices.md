---
title: "Slices"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Working with Collections", "Collections"]
related_syntax: ["[ ]"]
see_also: ["Arrays vs Vec"]
---

## Explanation

A slice, `[T]`, is a view into a contiguous run of elements without
owning them — almost always seen behind a reference, `&[T]` or
`&mut [T]`, since `[T]` itself is unsized (the compiler doesn't know its
length at compile time, so it can't be a plain stack value).

A slice reference is a "fat pointer": under the hood it's a pointer to
the first element plus a length, which is why slicing an array or `Vec`
(`&v[1..3]`) doesn't copy any elements — it just produces a new
pointer+length pair describing a sub-range of the original data. This
makes slices the natural common interface for "a sequence of `T`" that
works identically whether the backing storage is a fixed-size array, a
`Vec`, or a sub-range of either — a function taking `&[T]` accepts all
three without needing to know or care which one it was handed.

Bounds are checked at runtime on indexing and slicing (`arr[5]`,
`arr[1..3]`) — an out-of-range access panics rather than reading
adjacent memory, which is part of what makes slices memory-safe despite
being a thin, low-level view rather than an owning collection.

## Basic usage example

```
let v = vec![10, 20, 30, 40];
let s: &[i32] = &v[1..3]; // <- a view into part of v; no elements are copied
println!("{:?}", s);       // [20, 30]
```

**Restriction:** indexing or slicing out of range panics at runtime
(`&v[1..10]` here would panic) rather than being caught at compile time
— use `.get(range)`, which returns `Option`, when the bounds aren't
already known to be valid.

## Embedded Rust Notes

**Full support.** `[T]` lives in `core` — no allocator needed. Slices
over statically-sized buffers (`&mut [u8]` for a DMA transfer, for
instance) are a staple of allocator-free embedded code.
