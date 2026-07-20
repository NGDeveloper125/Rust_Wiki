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

Both produce a [slice](slices.md) reference `&[T]` for reading — `Vec`
via a `Deref` impl, an array via an unsized coercion — which is why
functions that just need to *read* a sequence, regardless of its backing
storage, conventionally take `&[T]` rather than committing to either
`&Vec<T>` or `&[T; N]` specifically.

## Basic usage example

```
let arr: [i32; 3] = [1, 2, 3];   // <- fixed length baked into the type, stack-allocated

let mut v: Vec<i32> = vec![1, 2, 3];
v.push(4);                        // <- Vec can grow at runtime; arr's length can never change
```

**Restriction:** an array's length is part of its type and fixed at
compile time — it cannot grow or shrink, and a length only known at
runtime (parsed input, for example) can't be expressed as `[T; N]` at
all, which is exactly when `Vec` is required instead.

## Best practices & deeper information

### Scenario: Working with collections

Reach for an array the moment a collection's length is a fixed fact of
the domain; reach for `Vec` the moment it depends on anything decided at
runtime.

```
let rgb: [u8; 3] = [255, 87, 34]; // <- length is always exactly 3: no heap allocation, can't grow

let mut samples: Vec<f64> = Vec::new(); // <- length isn't known ahead of time
for reading in [21.5, 22.0, 21.8] {     // stand-in for readings arriving one at a time
    samples.push(reading); // <- Vec can grow; an array's length could never change like this
}
```

**Why this way:** `Vec` is the right default for the "length depends on
runtime input" case — an array only works when the count really is a
compile-time constant, like a color's three fixed channels.

### Scenario: Designing a public API

Whether a function signature takes `[T; N]` or `&[T]`/`Vec<T>` should
reflect whether the length is a real, fixed contract of the domain or
something the caller determines.

```
fn checksum(bytes: [u8; 4]) -> u8 { // <- fixed-size array: caller must supply exactly 4 bytes
    bytes.iter().fold(0, |acc, b| acc ^ b)
}

fn checksum_stream(bytes: &[u8]) -> u8 { // <- slice: any length, decided by the caller at runtime
    bytes.iter().fold(0, |acc, b| acc ^ b)
}

checksum([0x1, 0x2, 0x3, 0x4]);
checksum_stream(&vec![0x1, 0x2, 0x3, 0x4, 0x5]);
```

**Why this way:** if a protocol genuinely fixes the length — a 4-byte
header, say — saying so in the signature with `[T; N]` turns a
wrong-length call into a compile error instead of a runtime bounds
check; only widen to `&[T]`/`Vec<T>` once the length is truly
caller-determined.

## Embedded Rust Notes

**Partial support.** Fixed-size arrays (`[T; N]`) are full support — pure
`core`, stack-allocated, no allocator needed, and the default choice in
embedded code for exactly that reason. `Vec<T>` itself lives in `alloc`
and needs a configured `#[global_allocator]`; where growable
storage is needed without a heap, `heapless::Vec<T, N>` provides a
fixed-*capacity* (but runtime-variable-*length*) alternative with no
allocator dependency at all.
