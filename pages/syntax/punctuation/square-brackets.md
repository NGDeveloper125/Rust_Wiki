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
see the [Arrays vs Vec](../../concepts/types-data-modeling/arrays-vs-vec.md) concept page.

## Usage examples

### Declaring array types, indexing, and slicing

```
let arr: [i32; 3] = [1, 2, 3]; // <- `[i32; 3]` is the type, `[1, 2, 3]` the literal
let first = arr[0];            // <- `[0]` here is indexing
let slice: &[i32] = &arr[1..3]; // <- `[1..3]` here is slicing
```

**Restriction:** both indexing and slicing panic at runtime if the
index/range is out of bounds — there is no `Option`-returning `[]` form
(use `.get()` for that instead).

### Working with collections

`.get(i)` returns `Option<&T>` instead of panicking, which makes it the
right default for indices that aren't already known to be in bounds —
`arr[i]` is best reserved for cases where an out-of-bounds index would
itself be a bug worth crashing on.

```
let scores = [85, 92, 78];

// AVOID: arr[i] panics the whole program if `i` ever comes from untrusted input
let third = scores[2];

// PREFER: .get(i) turns an out-of-range index into a normal Option to handle
match scores.get(5) {
    Some(score) => println!("{score}"),
    None => println!("no 6th score"),
}
```

`[]` indexing is appropriate when an out-of-range index
means the program's own logic is broken (a genuine bug, worth a panic);
`.get()` is appropriate whenever the index could legitimately be
out-of-range because of external input — the
[std docs for slice indexing](https://doc.rust-lang.org/std/primitive.slice.html#method.get)
document this as the alternative specifically to avoid the panic.

### Sharing data with multiple references

Slicing with `[ ]` doesn't copy — `&arr[1..3]` borrows a view into the
original array/`Vec`, so several slices of the same data can coexist
under the normal shared-borrow rules.

```
let data = vec![10, 20, 30, 40, 50];
let head = &data[..2];  // <- borrows the first two elements
let tail = &data[2..];  // <- borrows the rest; both slices borrow `data` at once

println!("{head:?} {tail:?}"); // fine: both are read-only shared borrows
```

Because a slice is just a `(pointer, length)` view, not
an allocation, splitting data into overlapping or adjacent read-only
views via `[ ]` is effectively free — no cloning needed, unlike languages
where a "slice" implies a copy.

## Explanation (Embedded)

`[ ]` means exactly the same thing under `#![no_std]` — array/slice
types, literals, and indexing all live in `core`, with no `std`
dependency. Fixed-size arrays are, if anything, more central in embedded
code than in hosted code: without a heap, `[T; N]` (or a
`heapless::Vec<T, N>` built on top of one) is usually the *only* buffer
type available, standing in for the `Vec<T>` a hosted program would reach
for by default.

## Usage examples (Embedded)

### A fixed-size buffer standing in for `Vec`

```
#![no_std]

use heapless::Vec;

let mut samples: Vec<u16, 16> = Vec::new(); // <- backed by a `[u16; 16]`, not a heap allocation
samples.push(512).ok();
```

### Slicing a DMA receive buffer

```
let rx_buf: [u8; 64] = [0; 64];
let header = &rx_buf[..4]; // <- `[..4]` slices the first 4 bytes without copying
```
