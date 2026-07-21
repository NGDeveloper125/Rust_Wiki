---
title: "vec!"
kind: macro
embedded_support: partial
groups: ["Collections", "Macros & Metaprogramming"]
related_concepts: ["Vec<T>"]
related_syntax: ["[ ]", "!"]
see_also: ["Vec<T>"]
---

## Explanation

`vec!` has two forms. The list form, `vec![1, 2, 3]`, expands to creating
an empty `Vec` and pushing each element in order, evaluating to the
result — the same outcome as `Vec::new()` plus a `push` per element, just
shorter to write. The repeat form, `vec![element; count]`, instead builds
a `Vec` of `count` elements each equal to `element` — echoing the
array-repeat literal `[element; count]`'s syntax, but the two are not the
same operation underneath.

An array repeat literal `[0; 10]` can, for an element the compiler
recognizes as a constant, be evaluated once at compile time and copied
into place. `vec![element; count]` never has that option, since a `Vec`'s
buffer is always heap-allocated at runtime. Instead, `vec![element;
count]` requires `element: Clone` (not merely `Copy`) and desugars to
allocating a buffer of `count` slots and calling `.clone()` to fill them
— the compiler special-cases genuinely `Copy` types to memcpy the value
across the buffer, but for a non-`Copy` type (a `String`, a `Vec`, a
struct owning either) that's a real runtime loop, one clone per slot.
This is exactly why `vec![String::new(); n]` compiles where the plain
array form `[String::new(); n]` does not: arrays require `Copy` for their
repeat form outright, while `vec!` only requires `Clone`, at the cost of
the per-element cloning actually happening at runtime.

## Usage examples

### Building a Vec with the list and repeat forms

```
let ids = vec![101, 102, 103]; // <- list form: three explicit elements
let zeros = vec![0u8; 8];      // <- repeat form: 8 copies of 0u8
```

**Restriction:** the repeat count in `vec![element; count]` must be a
`usize` expression, and `element` must implement `Clone` —
`vec![Mutex::new(0); 4]` doesn't compile, since `Mutex` isn't `Clone`
(four independent mutexes can't be produced by cloning one).

### Working with collections

Pre-sizing a lookup table of per-sensor accumulators is the natural job
for the repeat form — "N copies of a starting value" — rather than a
manual push loop.

```
const SENSOR_COUNT: usize = 6;

let totals = vec![0.0_f64; SENSOR_COUNT];         // <- repeat form: SENSOR_COUNT independent f64 accumulators, all 0.0
let labels = vec!["temp", "humidity", "pressure"]; // <- list form: explicit, known-up-front elements

println!("{totals:?}");
println!("{labels:?}");
```

The
[std Vec docs](https://doc.rust-lang.org/std/vec/struct.Vec.html) treat
`vec![value; n]` as the idiomatic spelling for "N slots, each starting
equal" — allocated and filled in one call rather than
`Vec::with_capacity(n)` plus a manual fill loop.

### Creating a new object

Building a fixed-size grid of independently owned buffers — each row its
own `Vec<u8>` — is exactly where `Clone`, not `Copy`, is what makes the
repeat form legal at all.

```
const ROWS: usize = 4;
const COLS: usize = 16;

let grid: Vec<Vec<u8>> = vec![vec![0u8; COLS]; ROWS];
// <- the outer vec! clones the inner Vec<u8> (non-Copy) ROWS times — a real runtime clone per row, not a memcpy
```

Reaching for `vec![inner; n]` here is correct
specifically because each row must be its own independently owned
allocation — cloning is the operation the code actually needs, not an
accidental cost; see [`Vec<T>`](../../concepts/collections-strings/vec.md)
for how the resulting `Vec` behaves once built.

## Explanation (Embedded)

`vec!` is defined purely in terms of `alloc::vec::Vec` — under
`#![no_std]` it's available exactly when the `alloc` crate is pulled in
and a `#[global_allocator]` is configured to back it, at which point
`vec![1, 2, 3]` and `vec![0u8; 8]` behave identically to a hosted build:
same list/repeat forms, same `Clone`-not-`Copy` requirement for the
repeat form. On a target with no heap at all — no allocator configured,
no `alloc` — `vec!` simply isn't available, and there is no
macro-level equivalent to reach for instead:
[`heapless::Vec<T, N>`](../../concepts/collections-strings/vec.md) (a
fixed-capacity, statically allocated vector) is the idiomatic substitute,
but it's built imperatively — `heapless::Vec::new()` followed by
`.push()` per element, or `.resize()` to fill it to a starting value —
rather than through a `vec!`-style literal, since a fixed-capacity
collection has no single expression that means "however many elements,
however big the source list turns out to be."

## Usage examples (Embedded)

### Building a Vec once alloc is configured

```
extern crate alloc;
use alloc::vec::Vec;

fn make_thresholds() -> Vec<u16> {
    vec![100, 200, 300] // <- list form, identical to hosted Rust once `alloc` + a #[global_allocator] are set up
}
```

### Building a fixed-capacity buffer with heapless (no allocator)

```
use heapless::Vec;

fn make_readings() -> Vec<f32, 8> { // <- capacity 8, fixed at compile time, no heap involved
    let mut readings = Vec::new(); // <- heapless::Vec has no vec!-style literal
    readings.push(21.4).unwrap(); // <- push() returns a Result, since the buffer can be full
    readings.push(19.8).unwrap();
    readings
}
```

### Pre-filling a heapless buffer to a starting value

Where classic code reaches for `vec![0u8; 8]`'s repeat form, the
`heapless` equivalent is a `.resize()` call on an already-constructed
fixed-capacity `Vec`.

```
use heapless::Vec;

fn zeroed_frame() -> Vec<u8, 8> {
    let mut frame = Vec::new();
    frame.resize(8, 0u8).unwrap(); // <- fills to 8 elements of 0u8, the resize() this page's vec![elem; n] maps onto
    frame
}
```
