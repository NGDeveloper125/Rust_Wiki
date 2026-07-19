---
title: "[ ]"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Arrays vs Vec, Slices]
related_syntax: [";"]
see_also: [";"]
---

## Explanation

`[ ]` covers array/slice construction, array/slice types, and indexing:

- **Array literal:** `[1, 2, 3]` — a fixed-size array value.
- **Array repeat literal:** `[0; 10]` — ten copies of `0`; the part after
  `;` must be a `const`-evaluable length.
- **Array type:** `[i32; 10]` — a fixed-size array type, length included.
- **Slice type:** `[i32]` — an *unsized* type (no length in the type
  itself); almost always seen behind a reference, as `&[i32]` or
  `&mut [i32]`.
- **Indexing:** `arr[0]` — calls the `Index`/`IndexMut` trait; panics on
  out-of-bounds access rather than returning an `Option`.
- **Slicing:** `arr[1..3]`, `arr[..2]`, `arr[2..]`, `arr[..]` — produces a
  sub-slice using a range as the index; also panics if the range is out
  of bounds.

The distinction between an array type `[T; N]` (size is part of the type,
known at compile time) and a slice type `[T]` (size is not part of the
type, checked at runtime) is a frequent point of confusion for newcomers —
see the [Arrays vs Vec](../../concepts/arrays-vs-vec.md) concept page.

## Basic usage example

```
let arr: [i32; 3] = [1, 2, 3]; // <- `[i32; 3]` is the type, `[1, 2, 3]` the literal
let first = arr[0];            // <- `[0]` here is indexing
let slice: &[i32] = &arr[1..3]; // <- `[1..3]` here is slicing
```

**Restriction:** both indexing and slicing panic at runtime if the
index/range is out of bounds — there is no `Option`-returning `[]` form
(use `.get()` for that instead).

## Embedded Rust Notes

**Full support.** Array and slice syntax is core grammar (`[T; N]` and
`[T]` both live in `core`) — no `std` dependency, and fixed-size arrays
are the single most-used data structure in allocator-free embedded code.
