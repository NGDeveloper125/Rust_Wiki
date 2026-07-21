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

## Explanation (Embedded)

An array, `[T; N]`, needs no allocator at all — its length is baked into
the type, its storage is inline (on the stack, or in `static` memory for a
`'static` array), and there is nothing for `alloc` or a
`#[global_allocator]` to do. That makes `[T; N]` the default embedded
choice whenever a collection's size is a fixed fact known at compile
time: a 3-axis accelerometer reading, a fixed-width protocol frame, a
lookup table sized to a peripheral's channel count.

`Vec<T>` needs the opposite: it lives in `alloc`, not `core`, so it only
compiles once a crate pulls in `alloc` and wires up a
`#[global_allocator]` — see
[`Vec<T>`'s embedded section](../collections-strings/vec.md) for that
setup and for `heapless::Vec<T, N>`, the fixed-*capacity*,
no-allocator substitute that gives back `Vec`-like ergonomics (`.push()`,
`.pop()`, a runtime-tracked `.len()` up to a compile-time-fixed bound)
without ever touching a heap.

An array is strictly more restrictive than either — its length can never
change, full stop, where `heapless::Vec<T, N>` at least lets the *length*
vary at runtime up to `N` — but that restriction is also the array's
whole appeal on constrained hardware: no capacity check on push (there is
no push), no `Result` to handle for "buffer full," and not even
`heapless`'s small bookkeeping overhead of a runtime length field. When
the size truly never varies, an array is the zero-dependency option, and
`heapless::Vec<T, N>` earns its keep specifically for the cases where the
count is bounded but genuinely variable at runtime.

## Basic usage example (Embedded)

```
let calibration: [i16; 3] = [12, -4, 7]; // <- fixed 3-axis offset, no allocator, no heapless dependency

fn apply_offset(raw: [i16; 3], offset: [i16; 3]) -> [i16; 3] {
    [raw[0] + offset[0], raw[1] + offset[1], raw[2] + offset[2]]
}

let corrected = apply_offset([100, 200, -50], calibration);
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

A calibration table with a fixed, datasheet-defined number of channels
should be an array; a log of readings collected until a buffer fills, one
at a time, is the case `heapless::Vec` is actually for.

```
let channel_gains: [f32; 4] = [1.02, 0.98, 1.00, 1.01]; // <- always exactly 4 channels: no heap, no bookkeeping

use heapless::Vec;
let mut recent: Vec<f32, 16> = Vec::new(); // <- fixed capacity, but a runtime length that grows as readings arrive
for reading in [21.5, 22.0, 21.8] {
    recent.push(reading).ok(); // <- Err once the buffer is full, handled explicitly instead of reallocating
}
```

**Why this way:** the channel count is a hardware fact that never
changes across the program's life, so an array needs nothing beyond
`core`; the reading log's length is decided by how many samples have
arrived so far, which is exactly the runtime-variable-length,
fixed-capacity case `heapless::Vec` exists for — see its
[embedded section](../collections-strings/vec.md) for the capacity-bound
tradeoff in more depth.

### Scenario: Designing a public API

A driver function that reads a protocol-fixed number of bytes should say
so with `[u8; N]` in its signature; a function reading a variable-length
frame up to a known maximum should take `&mut [u8]` and return how much
of it was actually filled.

```
fn read_device_id(bytes: [u8; 2]) -> u16 { // <- protocol always sends exactly 2 bytes: no allocator needed
    u16::from_be_bytes(bytes)
}

fn read_frame(buf: &mut [u8]) -> usize { // <- caller's buffer, any backing storage, any length up to buf.len()
    buf[0] = 0xAA;
    1 // number of bytes actually written
}

let id = read_device_id([0x01, 0x2C]);
let mut frame = [0u8; 32]; // stack-allocated, no heap
let written = read_frame(&mut frame);
```

**Why this way:** a fixed-size array in the signature turns a
wrong-length call into a compile error, which matters more on hardware
where there's no test suite running against every possible caller — while
a variable-length read has no honest way to express its bound as a type,
so it takes a slice and reports its actual length back, the same
allocator-free contract a DMA-backed buffer already needs.
